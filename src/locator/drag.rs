use crate::error::Error;
use crate::human::Point;
use crate::input::{Modifiers, MouseButton, Pointer, Press};
use crate::session::Session;
use serde_json::{Value, json};
use std::time::Duration;
use tokio::time::Instant;

const DRAG_STEPS: u32 = 10;

impl Session {
    pub(super) async fn drag(
        &mut self,
        from: (f64, f64),
        to: (f64, f64),
        html5: bool,
    ) -> Result<(), Error> {
        let press = Press::new(MouseButton::Left, Modifiers::NONE);
        self.pointer(Pointer::Move, from.0, from.1, press).await?;
        self.intercept_drags(html5, true).await?;
        self.pointer(Pointer::Down, from.0, from.1, press).await?;
        let data = self.drag_along(from, to, html5, press).await?;
        let result = match &data {
            Some(data) => self.drop_at(to, data).await,
            None => self.pointer(Pointer::Up, to.0, to.1, press).await,
        };
        self.intercept_drags(html5, false).await?;
        result
    }

    async fn drag_along(
        &mut self,
        from: (f64, f64),
        to: (f64, f64),
        html5: bool,
        press: Press,
    ) -> Result<Option<Value>, Error> {
        let mut data = None;
        for (x, y, after) in self.drag_points(from, to) {
            self.pointer(Pointer::Drag, x, y, press).await?;
            data = self.drag_step(html5, data, (x, y)).await?;
            self.idle(after).await?;
        }
        if html5 && data.is_none() {
            return self.intercepted_drag(self.timing().actionable).await;
        }
        Ok(data)
    }

    async fn drag_step(
        &mut self,
        html5: bool,
        data: Option<Value>,
        at: (f64, f64),
    ) -> Result<Option<Value>, Error> {
        if let Some(data) = data {
            self.drag_event("dragOver", at, &data).await?;
            return Ok(Some(data));
        }
        if !html5 {
            return Ok(None);
        }
        let found = self.intercepted_drag(Duration::ZERO).await?;
        if let Some(data) = &found {
            self.drag_event("dragEnter", at, data).await?;
        }
        Ok(found)
    }

    async fn drop_at(&mut self, at: (f64, f64), data: &Value) -> Result<(), Error> {
        self.drag_event("dragEnter", at, data).await?;
        self.drag_event("dragOver", at, data).await?;
        self.drag_event("drop", at, data).await
    }

    async fn intercept_drags(&mut self, html5: bool, enabled: bool) -> Result<(), Error> {
        if html5 {
            let params = json!({ "enabled": enabled });
            drop(self.send("Input.setInterceptDrags", params).await?);
        }
        Ok(())
    }

    async fn intercepted_drag(&mut self, wait: Duration) -> Result<Option<Value>, Error> {
        let deadline = Instant::now() + wait;
        loop {
            let found = self
                .take_event(|event| {
                    event.get("method").and_then(Value::as_str) == Some("Input.dragIntercepted")
                })
                .await?;
            if let Some(event) = found {
                return Ok(event.pointer("/params/data").cloned());
            }
            if Instant::now() >= deadline {
                return Ok(None);
            }
            self.idle(self.timing().poll).await?;
        }
    }

    async fn drag_event(&mut self, kind: &str, at: (f64, f64), data: &Value) -> Result<(), Error> {
        let params = json!({ "type": kind, "x": at.0.round(), "y": at.1.round(), "data": data });
        drop(self.send("Input.dispatchDragEvent", params).await?);
        Ok(())
    }

    fn drag_points(&mut self, from: (f64, f64), to: (f64, f64)) -> Vec<(f64, f64, Duration)> {
        let start = Point {
            x: from.0,
            y: from.1,
        };
        let end = Point { x: to.0, y: to.1 };
        if let Some(steps) = self.human_path(start, end) {
            return steps
                .into_iter()
                .map(|step| (step.point.x, step.point.y, step.after))
                .collect();
        }
        (1..=DRAG_STEPS)
            .map(|i| {
                let t = f64::from(i) / f64::from(DRAG_STEPS);
                (
                    from.0 + (to.0 - from.0) * t,
                    from.1 + (to.1 - from.1) * t,
                    Duration::ZERO,
                )
            })
            .collect()
    }
}
