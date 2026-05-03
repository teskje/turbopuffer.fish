use std::sync::OnceLock;

use wasm_bindgen::prelude::*;
use web_sys::{CanvasGradient, CanvasRenderingContext2d, HtmlCanvasElement};

use crate::Vec2;

const FONT_FAMILY: &str = "JuliaMono";

static CHAR_WIDTH_RATIO: OnceLock<f64> = OnceLock::new();

pub(crate) fn char_width(font_size: f64) -> f64 {
    CHAR_WIDTH_RATIO.get().expect("canvas must be initialized") * font_size
}

pub(crate) struct Canvas {
    el: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    pub(crate) dim: Vec2,
}

impl Canvas {
    pub(crate) fn new(el: HtmlCanvasElement) -> Result<Self, JsValue> {
        let ctx = el
            .get_context("2d")?
            .ok_or("no 2d context")?
            .dyn_into::<CanvasRenderingContext2d>()?;

        let mut canvas = Canvas {
            el,
            ctx,
            dim: Default::default(),
        };
        canvas.resize()?;

        let measure_size = 100.0;
        canvas.set_font_size(measure_size);
        let measured = canvas.ctx.measure_text("M")?.width();
        let _ = CHAR_WIDTH_RATIO.set(measured / measure_size);

        Ok(canvas)
    }

    pub(crate) fn resize(&mut self) -> Result<(), JsValue> {
        let window = web_sys::window().ok_or("no window")?;
        let w = window.inner_width()?.as_f64().expect("fits");
        let h = window.inner_height()?.as_f64().expect("fits");
        let dpr = window.device_pixel_ratio();

        self.dim = Vec2::new(w, h);

        // HiDPI: canvas pixel size is scaled up, CSS size stays logical
        self.el.set_width((w * dpr) as u32);
        self.el.set_height((h * dpr) as u32);
        self.el.style().set_property("width", &format!("{w}px"))?;
        self.el.style().set_property("height", &format!("{h}px"))?;
        self.ctx.scale(dpr, dpr)?;
        Ok(())
    }

    pub(crate) fn set_font_size(&self, size: f64) {
        self.ctx.set_font(&format!("{size}px {FONT_FAMILY}"));
    }

    pub(crate) fn set_font(&self, size: f64, weight: &str) {
        self.ctx
            .set_font(&format!("{weight} {size}px {FONT_FAMILY}"));
    }

    pub(crate) fn set_color(&self, color: &str) {
        self.ctx.set_fill_style(&JsValue::from_str(color));
    }

    pub(crate) fn fill_text(&self, text: &str, pos: Vec2) {
        let _ = self.ctx.fill_text(text, pos.x, pos.y);
    }

    pub(crate) fn scale(&self, x: f64, y: f64) {
        let _ = self.ctx.scale(x, y);
    }

    pub(crate) fn with_save(&self, f: impl FnOnce(&Canvas)) {
        self.ctx.save();
        f(self);
        self.ctx.restore();
    }

    pub(crate) fn clear_with_gradient(&self, f: impl FnOnce(&CanvasGradient)) {
        let gradient = self.ctx.create_linear_gradient(0.0, 0.0, 0.0, self.dim.y);
        f(&gradient);
        self.ctx.set_fill_style(&gradient);
        self.ctx.fill_rect(0.0, 0.0, self.dim.x, self.dim.y);
    }
}
