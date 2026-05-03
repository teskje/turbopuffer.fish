use crate::bubble::Bubble;
use crate::canvas::Canvas;
use crate::puffer::Puffer;
use crate::{Rect, Vec2};

pub enum Entity {
    Puffer(Puffer),
    Bubble(Bubble),
}

impl Entity {
    pub fn bounds(&self) -> Rect {
        match self {
            Entity::Puffer(p) => p.bounds(),
            Entity::Bubble(b) => b.bounds(),
        }
    }

    pub fn on_hover(&mut self) {
        match self {
            Entity::Puffer(p) => p.startle(),
            Entity::Bubble(_) => {}
        }
    }

    pub fn update(&mut self, delta_s: f64, world_dim: Vec2) -> Vec<Entity> {
        match self {
            Entity::Puffer(p) => p.update(delta_s, world_dim),
            Entity::Bubble(b) => {
                b.update(delta_s);
                Vec::new()
            }
        }
    }

    pub fn is_alive(&self) -> bool {
        match self {
            Entity::Puffer(_) => true,
            Entity::Bubble(b) => b.is_alive(),
        }
    }

    pub fn render(&self, canvas: &Canvas) {
        match self {
            Entity::Puffer(p) => p.render(canvas),
            Entity::Bubble(b) => b.render(canvas),
        }
    }

    pub fn on_click(&mut self) {
        match self {
            Entity::Puffer(p) => p.puff(),
            Entity::Bubble(b) => b.pop(),
        }
    }

    pub fn collide(&mut self, other: &Self) {
        match self {
            Entity::Puffer(p) => p.collide(other),
            Entity::Bubble(b) => b.collide(other),
        }
    }
}
