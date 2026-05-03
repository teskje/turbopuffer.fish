mod bubble;
mod canvas;
mod entity;
mod palette;
mod puffer;

use std::ops::{Add, AddAssign, Mul, MulAssign};

use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

use crate::canvas::Canvas;
use crate::entity::Entity;
use crate::puffer::Puffer;

const PUFFER_COUNT: usize = 20;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

macro_rules! console_error {
    ($($t:tt)*) => (error(&format_args!($($t)*).to_string()))
}

fn set_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        console_error!("{}", info);
    }));
}

#[wasm_bindgen]
pub struct Game {
    canvas: Canvas,
    entities: Vec<Entity>,
    last_frame_time_ms: f64,
}

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_el: HtmlCanvasElement) -> Result<Game, JsValue> {
        set_panic_hook();

        let canvas = Canvas::new(canvas_el)?;

        let mut entities = Vec::new();
        for _ in 0..PUFFER_COUNT {
            let puffer = Puffer::new_random(canvas.dim);
            entities.push(Entity::Puffer(puffer));
        }

        console_log!("turbopuffer.fish: {} entities spawned", entities.len());

        Ok(Game {
            canvas,
            entities,
            last_frame_time_ms: 0.0,
        })
    }

    pub fn tick(&mut self, time_ms: f64) {
        let delta_ms = (time_ms - self.last_frame_time_ms).clamp(0., 100.);
        let delta_s = delta_ms / 1000.;
        self.last_frame_time_ms = time_ms;

        let mut spawned = Vec::new();
        for entity in &mut self.entities {
            spawned.extend(entity.update(delta_s, self.canvas.dim));
        }
        self.entities.extend(spawned);

        self.entities.retain(|e| e.is_alive());
        self.resolve_collisions();
        self.render();
    }

    pub fn on_click(&mut self, x: f64, y: f64) -> bool {
        let point = Vec2 { x, y };
        let mut hit = false;
        for entity in &mut self.entities {
            if entity.bounds().contains(point) {
                entity.on_click();
                hit = true;
            }
        }
        hit
    }

    pub fn on_hover(&mut self, x: f64, y: f64) -> bool {
        let point = Vec2 { x, y };
        let mut hit = false;
        for entity in &mut self.entities {
            if entity.bounds().contains(point) {
                entity.on_hover();
                hit = true;
            }
        }
        hit
    }

    pub fn on_resize(&mut self) {
        let _ = self.canvas.resize();
    }

    fn resolve_collisions(&mut self) {
        // Sweep-and-prune: sort by left edge, skip pairs that can't overlap on x.
        self.entities.sort_unstable_by(|a, b| {
            let xa = a.bounds().min.x;
            let xb = b.bounds().min.x;
            xa.total_cmp(&xb)
        });

        let n = self.entities.len();
        for i in 0..n.saturating_sub(1) {
            let a_bounds = self.entities[i].bounds();
            let a_right = a_bounds.max.x;

            for j in (i + 1)..n {
                let b_bounds = self.entities[j].bounds();
                let b_left = b_bounds.min.x;
                if b_left >= a_right {
                    break;
                }

                if a_bounds.overlaps(b_bounds) {
                    let (left, right) = self.entities.split_at_mut(j);
                    left[i].collide(&right[0]);
                    right[0].collide(&left[i]);
                }
            }
        }
    }

    fn render(&self) {
        self.canvas.clear_with_gradient(|gradient| {
            let _ = gradient.add_color_stop(0.0, palette::OCEAN_SURFACE);
            let _ = gradient.add_color_stop(0.2, palette::OCEAN_MID);
            let _ = gradient.add_color_stop(0.8, palette::OCEAN_DEEP);
        });

        for entity in &self.entities {
            entity.render(&self.canvas);
        }
    }
}

#[derive(Clone, Copy, Default)]
struct Vec2 {
    x: f64,
    y: f64,
}

impl Vec2 {
    fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

impl Add<Vec2> for Vec2 {
    type Output = Self;

    fn add(mut self, rhs: Vec2) -> Self::Output {
        self += rhs;
        self
    }
}

impl AddAssign<Vec2> for Vec2 {
    fn add_assign(&mut self, rhs: Vec2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Mul<f64> for Vec2 {
    type Output = Self;

    fn mul(mut self, rhs: f64) -> Self {
        self *= rhs;
        self
    }
}

impl MulAssign<f64> for Vec2 {
    fn mul_assign(&mut self, rhs: f64) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

#[derive(Clone, Copy)]
struct Rect {
    min: Vec2,
    max: Vec2,
}

impl Rect {
    fn contains(&self, point: Vec2) -> bool {
        self.overlaps(Rect {
            min: point,
            max: point,
        })
    }

    fn overlaps(&self, other: Rect) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }
}
