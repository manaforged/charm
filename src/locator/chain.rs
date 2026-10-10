use super::Locator;
use crate::selector::{Filter, Position, Selector};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ActionOptions {
    pub force: bool,
    pub timeout: Option<Duration>,
}

impl Locator<'_> {
    fn wrap(mut self, build: impl FnOnce(Box<Selector>) -> Selector) -> Self {
        let selector = std::mem::replace(&mut self.selector, Selector::Css(String::new()));
        let base = match self.index.take() {
            Some(index) => Selector::Nth {
                base: Box::new(selector),
                at: Position::Index(index),
            },
            None => selector,
        };
        self.selector = build(Box::new(base));
        self.context = None;
        self
    }

    #[must_use]
    pub fn locator(self, inner: impl Into<Selector>) -> Self {
        let inner = Box::new(inner.into());
        self.wrap(|outer| Selector::Within { outer, inner })
    }

    #[must_use]
    pub fn filter(self, filter: Filter) -> Self {
        let filter = Box::new(filter);
        self.wrap(|base| Selector::Filtered { base, filter })
    }

    #[must_use]
    pub fn and(self, other: impl Into<Selector>) -> Self {
        let other = Box::new(other.into());
        self.wrap(|base| Selector::And(base, other))
    }

    #[must_use]
    pub fn or(self, other: impl Into<Selector>) -> Self {
        let other = Box::new(other.into());
        self.wrap(|base| Selector::Or(base, other))
    }

    #[must_use]
    pub fn last(self) -> Self {
        self.wrap(|base| Selector::Nth {
            base,
            at: Position::Last,
        })
    }

    #[must_use]
    pub fn with_options(mut self, options: ActionOptions) -> Self {
        self.options = options;
        self
    }
}
