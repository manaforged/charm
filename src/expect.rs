use crate::dom::Read;
use crate::error::Error;
use crate::locator::Locator;
use crate::selector::{Selector, TextMatch, quote};
use crate::session::Session;
use std::ops::Not;

const NORM: &str = "(s=>String(s).replace(/\\s+/g,' ').trim())";

fn describe(matcher: &TextMatch) -> String {
    match matcher {
        TextMatch::Exact(text) => quote(text),
        TextMatch::Contains(text) => format!("containing {}", quote(text)),
        TextMatch::Regex { pattern, flags } => format!("/{pattern}/{flags}"),
    }
}

fn test(matcher: &TextMatch, value: &str, normalize: bool, contains: bool) -> String {
    let subject = if normalize {
        format!("{NORM}({value})")
    } else {
        format!("String({value})")
    };
    let wanted = |text: &str| {
        if normalize {
            format!("{NORM}({})", quote(text))
        } else {
            quote(text)
        }
    };
    match matcher {
        TextMatch::Regex { pattern, flags } => {
            format!(
                "new RegExp({},{}).test({subject})",
                quote(pattern),
                quote(flags)
            )
        }
        TextMatch::Contains(text) => format!("{subject}.includes({})", wanted(text)),
        TextMatch::Exact(text) if contains => format!("{subject}.includes({})", wanted(text)),
        TextMatch::Exact(text) => format!("{subject}==={}", wanted(text)),
    }
}

struct Check {
    expected: String,
    value: String,
    pass: String,
}

impl Check {
    fn probe(&self, want: bool) -> String {
        format!(
            "(()=>{{const v={};return ({})==={want};}})()",
            self.value, self.pass
        )
    }

    fn failure(self, want: bool, received: String, selector: Option<Selector>) -> Error {
        let expected = if want {
            self.expected
        } else {
            format!("not {}", self.expected)
        };
        Error::Assertion {
            expected,
            received,
            selector,
        }
    }
}

pub fn expect<'l, 's>(locator: &'l mut Locator<'s>) -> LocatorExpect<'l, 's> {
    LocatorExpect {
        locator,
        want: true,
    }
}

pub fn expect_page(session: &mut Session) -> PageExpect<'_> {
    PageExpect {
        session,
        want: true,
    }
}

pub struct LocatorExpect<'l, 's> {
    locator: &'l mut Locator<'s>,
    want: bool,
}

impl Not for LocatorExpect<'_, '_> {
    type Output = Self;

    fn not(mut self) -> Self {
        self.want = !self.want;
        self
    }
}

impl LocatorExpect<'_, '_> {
    fn flag(&self, read: &Read<'_>, expected: &str, holds: bool) -> Check {
        Check {
            expected: expected.to_owned(),
            value: read.expr(&self.locator.element_expr()),
            pass: if holds { "v===true" } else { "v===false" }.to_owned(),
        }
    }

    fn text(
        &self,
        read: &Read<'_>,
        name: &str,
        matcher: &TextMatch,
        normalize: bool,
        contains: bool,
    ) -> Check {
        Check {
            expected: format!("{name} {}", describe(matcher)),
            value: read.expr(&self.locator.element_expr()),
            pass: format!("v!==null&&{}", test(matcher, "v", normalize, contains)),
        }
    }

    async fn run(self, check: Check) -> Result<(), Error> {
        let timeout = self.locator.timing().wait;
        match self.locator.poll(&check.probe(self.want), timeout).await {
            Ok(_) => Ok(()),
            Err(Error::Timeout(_)) => {
                let received = self.locator.value(&check.value).await?.to_string();
                let selector = Some(self.locator.selector().clone());
                Err(check.failure(self.want, received, selector))
            }
            Err(error) => Err(error),
        }
    }

    pub async fn to_be_visible(self) -> Result<(), Error> {
        let check = self.flag(&Read::Visible, "to be visible", true);
        self.run(check).await
    }

    pub async fn to_be_hidden(self) -> Result<(), Error> {
        let check = self.flag(&Read::Visible, "to be hidden", false);
        self.run(check).await
    }

    pub async fn to_be_enabled(self) -> Result<(), Error> {
        let check = self.flag(&Read::Enabled, "to be enabled", true);
        self.run(check).await
    }

    pub async fn to_be_disabled(self) -> Result<(), Error> {
        let check = self.flag(&Read::Enabled, "to be disabled", false);
        self.run(check).await
    }

    pub async fn to_be_checked(self) -> Result<(), Error> {
        let check = self.flag(&Read::Checked, "to be checked", true);
        self.run(check).await
    }

    pub async fn to_be_focused(self) -> Result<(), Error> {
        let check = self.flag(&Read::Focused, "to be focused", true);
        self.run(check).await
    }

    pub async fn to_be_attached(self) -> Result<(), Error> {
        let check = Check {
            expected: "to be attached".to_owned(),
            value: self.locator.count_expr(),
            pass: "v>0".to_owned(),
        };
        self.run(check).await
    }

    pub async fn to_have_text(self, expected: impl Into<TextMatch>) -> Result<(), Error> {
        let check = self.text(
            &Read::TextContent,
            "to have text",
            &expected.into(),
            true,
            false,
        );
        self.run(check).await
    }

    pub async fn to_contain_text(self, expected: impl Into<TextMatch>) -> Result<(), Error> {
        let check = self.text(
            &Read::TextContent,
            "to contain text",
            &expected.into(),
            true,
            true,
        );
        self.run(check).await
    }

    pub async fn to_have_value(self, expected: impl Into<TextMatch>) -> Result<(), Error> {
        let check = self.text(
            &Read::InputValue,
            "to have value",
            &expected.into(),
            false,
            false,
        );
        self.run(check).await
    }

    pub async fn to_have_attribute(
        self,
        name: &str,
        expected: impl Into<TextMatch>,
    ) -> Result<(), Error> {
        let label = format!("to have attribute {}", quote(name));
        let check = self.text(
            &Read::Attribute(name),
            &label,
            &expected.into(),
            false,
            false,
        );
        self.run(check).await
    }

    pub async fn to_have_count(self, count: usize) -> Result<(), Error> {
        let check = Check {
            expected: format!("to have count {count}"),
            value: self.locator.count_expr(),
            pass: format!("v==={count}"),
        };
        self.run(check).await
    }
}

pub struct PageExpect<'s> {
    session: &'s mut Session,
    want: bool,
}

impl Not for PageExpect<'_> {
    type Output = Self;

    fn not(mut self) -> Self {
        self.want = !self.want;
        self
    }
}

impl PageExpect<'_> {
    async fn page(self, value: &str, name: &str, matcher: TextMatch) -> Result<(), Error> {
        let check = Check {
            expected: format!("{name} {}", describe(&matcher)),
            value: value.to_owned(),
            pass: test(&matcher, "v", false, false),
        };
        let timeout = self.session.timing().wait;
        match self
            .session
            .poll_expression(&check.probe(self.want), timeout)
            .await
        {
            Ok(_) => Ok(()),
            Err(Error::Timeout(_)) => {
                let received = self.session.evaluate(&check.value).await?.to_string();
                Err(check.failure(self.want, received, None))
            }
            Err(error) => Err(error),
        }
    }

    pub async fn to_have_url(self, expected: impl Into<TextMatch>) -> Result<(), Error> {
        self.page("location.href", "page to have URL", expected.into())
            .await
    }

    pub async fn to_have_title(self, expected: impl Into<TextMatch>) -> Result<(), Error> {
        self.page("document.title", "page to have title", expected.into())
            .await
    }
}
