use super::Locator;
use crate::dom::{BoundingBox, Read};
use crate::error::Error;
use serde::de::DeserializeOwned;

impl Locator<'_> {
    async fn get<T: DeserializeOwned>(&mut self, read: Read<'_>) -> Result<T, Error> {
        let expr = read.expr(&self.selector.one_expr(self.index));
        self.eval(&expr).await
    }

    pub async fn bounding_box(&mut self) -> Result<Option<BoundingBox>, Error> {
        let local: Option<BoundingBox> = self.get(Read::BoundingBox).await?;
        let context = self.context().await?;
        Ok(local.map(|bounds| context.shift(bounds)))
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
