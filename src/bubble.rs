use js_sys::Math;

use crate::canvas::{self, Canvas};
use crate::entity::Entity;
use crate::{Rect, Vec2, palette};

const FONT_SIZE: f64 = 18.0;
const RISE_SPEED: f64 = 40.0;

const GLYPHS: &[&str] = &["·", "°", "○"];

pub struct Bubble {
    position: Vec2,
    speed: f64,
    glyph: &'static str,
    alive: bool,
}

impl Bubble {
    pub fn new(position: Vec2) -> Self {
        let speed = RISE_SPEED * (0.8 + Math::random() * 0.4);
        Bubble {
            position,
            speed,
            glyph: random_glyph(),
            alive: true,
        }
    }

    pub fn update(&mut self, delta_s: f64) {
        self.position.y -= self.speed * delta_s;
        if self.position.y + FONT_SIZE < 0.0 {
            self.alive = false;
        }
    }

    pub fn pop(&mut self) {
        self.alive = false;
    }

    pub fn is_alive(&self) -> bool {
        self.alive
    }

    pub fn render(&self, canvas: &Canvas) {
        canvas.set_font_size(FONT_SIZE);
        canvas.set_color(palette::BUBBLE);

        let baseline = self.position.y + FONT_SIZE;
        let pos = Vec2::new(self.position.x, baseline);
        canvas.fill_text(self.glyph, pos);
    }

    pub fn collide(&mut self, other: &Entity) {
        if matches!(other, Entity::Puffer(_)) {
            self.pop();
        }
    }

    pub fn bounds(&self) -> Rect {
        let w = canvas::char_width(FONT_SIZE);
        Rect {
            min: self.position,
            max: self.position + Vec2::new(w, FONT_SIZE),
        }
    }
}

fn random_glyph() -> &'static str {
    let i = (Math::random() * GLYPHS.len() as f64) as usize;
    GLYPHS[i]
}
