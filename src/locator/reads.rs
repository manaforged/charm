use super::Locator;
use crate::dom::{BoundingBox, Read};
use crate::error::Error;
use crate::session::Session;
use serde::de::DeserializeOwned;

impl Session {
    pub(crate) async fn read<T: DeserializeOwned>(&mut self, expression: &str) -> Result<T, Error> {
        Ok(serde_json::from_value(self.evaluate(expression).await?)?)
    }
}

impl Locator<'_> {
    async fn get<T: DeserializeOwned>(&mut self, read: Read<'_>) -> Result<T, Error> {
        let expr = read.expr(&self.selector.one_expr(self.index));
        self.session.read(&expr).await
    }

    pub async fn bounding_box(&mut self) -> Result<Option<BoundingBox>, Error> {
        self.get(Read::BoundingBox).await
    }

    pub async fn input_value(&mut self) -> Result<Option<String>, Error> {
        self.get(Read::InputValue).await
    }

    pub async fn is_visible(&mut self) -> Result<bool, Error> {
        self.get(Read::Visible).await
    }

    pub async fn text_content(&mut self) -> Result<Option<String>, Error> {
        self.get(Read::TextContent).await
    }

    pub async fn inner_text(&mut self) -> Result<Option<String>, Error> {
        self.get(Read::InnerText).await
    }

    pub async fn attribute(&mut self, name: &str) -> Result<Option<String>, Error> {
        self.get(Read::Attribute(name)).await
    }

    pub async fn is_enabled(&mut self) -> Result<bool, Error> {
        self.get(Read::Enabled).await
    }

    pub async fn is_checked(&mut self) -> Result<bool, Error> {
        self.get(Read::Checked).await
    }

    pub async fn is_focused(&mut self) -> Result<bool, Error> {
        self.get(Read::Focused).await
    }

    pub async fn is_attached(&mut self) -> Result<bool, Error> {
        self.get(Read::Attached).await
    }
}
