use std::f64::consts::TAU;

use js_sys::Math;

use crate::bubble::Bubble;
use crate::canvas::{self, Canvas};
use crate::entity::Entity;
use crate::{Rect, Vec2, palette};

pub const FONT_SIZE: f64 = 24.0;

const NORMAL_SPEED: f64 = 100.0;
const STARTLED_SPEED: f64 = 200.0;
const PUFFED_SPEED: f64 = 30.0;
const FIN_CYCLE_PERIOD: f64 = 0.1;

const STARTLE_DURATION_S: f64 = 1.0;
const PUFF_DURATION_S: f64 = 10.0;
const BUBBLE_CHANCE_PER_S: f64 = 0.5;

const NORMAL_BODY: &str = "(°O°)";
const STARTLED_BODY: &str = "(ˣOˣ)";
const NORMAL_BODY_CHARS: usize = 5;
const NORMAL_WIDTH_CHARS: usize = NORMAL_BODY_CHARS + 2;

const PUFFED_TOP: &str = " ⸜⧹╵⧸⸝ ";
const PUFFED_MID: &str = "<(°O°)>";
const PUFFED_BOT: &str = " ⸍⧸╷⧹⸌ ";
const PUFFED_WIDTH_CHARS: usize = 7;

/// Vertical paddle: fin y-offset in pixels
const FIN_OFFSETS: &[f64] = &[-5.0, 0.0, 5.0, 0.0];

#[derive(Clone, Copy)]
pub enum State {
    Normal,
    Startled,
    Puffed,
}

pub struct Puffer {
    /// Position in the world; top-left corner of the bounding box.
    position: Vec2,
    /// Movement direction and velocity.
    movement: Vec2,
    state: State,
    cooldown: f64,
    fin_timer: f64,
    fin_offset: usize,
}

impl Puffer {
    pub fn new_random(world_dim: Vec2) -> Self {
        let fish_w = NORMAL_WIDTH_CHARS as f64 * char_width();
        let fish_h = FONT_SIZE;
        let mut puffer = Puffer {
            position: Vec2 {
                x: Math::random() * (world_dim.x - fish_w),
                y: Math::random() * (world_dim.y - fish_h),
            },
            movement: Default::default(),
            state: State::Normal,
            cooldown: 0.,
            fin_timer: Math::random() * FIN_CYCLE_PERIOD,
            fin_offset: (Math::random() * FIN_OFFSETS.len() as f64) as usize,
        };
        puffer.reset_movement();
        puffer
    }

    pub fn update(&mut self, delta_s: f64, world_dim: Vec2) -> Vec<Entity> {
        match self.state {
            State::Normal => {}
            State::Startled | State::Puffed => {
                self.cooldown -= delta_s;
                if self.cooldown <= 0. {
                    self.state = State::Normal;
                    self.reset_movement();
                }
            }
        };

        self.position += self.movement * delta_s;

        self.update_fin(delta_s);
        self.bounce_at_edges(world_dim);
        self.spawn_bubbles(delta_s)
    }

    fn update_fin(&mut self, delta_s: f64) {
        let speed = self.speed();
        let speed_ratio = speed / NORMAL_SPEED;
        let fin_period = FIN_CYCLE_PERIOD / speed_ratio;

        self.fin_timer += delta_s;
        if self.fin_timer >= fin_period {
            self.fin_timer -= fin_period;
            self.fin_offset = (self.fin_offset + 1) % FIN_OFFSETS.len();
        }
    }

    fn bounce_at_edges(&mut self, world_dim: Vec2) {
        let dim = self.dimensions();
        if (self.position.x + dim.x < 0. && self.movement.x < 0.)
            || (self.position.x > world_dim.x && self.movement.x > 0.)
        {
            self.movement.x *= -1.;
        }
        if (self.position.y < 0. && self.movement.y < 0.)
            || (self.position.y + dim.y > world_dim.y && self.movement.y > 0.)
        {
            self.movement.y *= -1.;
        }
    }

    fn spawn_bubbles(&self, delta_s: f64) -> Vec<Entity> {
        let chance = match self.state {
            State::Normal => BUBBLE_CHANCE_PER_S,
            State::Startled => BUBBLE_CHANCE_PER_S * 10.0,
            State::Puffed => BUBBLE_CHANCE_PER_S * 5.0,
        };
        if Math::random() > chance * delta_s {
            return Vec::new();
        }

        let dim = self.dimensions();
        let bubble_pos = self.position + Vec2::new(dim.x * 0.5, -FONT_SIZE);
        let bubble = Bubble::new(bubble_pos);
        vec![Entity::Bubble(bubble)]
    }

    fn speed(&self) -> f64 {
        f64::hypot(self.movement.x, self.movement.y)
    }

    fn reset_movement(&mut self) {
        let base_speed = match self.state {
            State::Normal => NORMAL_SPEED,
            State::Startled => STARTLED_SPEED,
            State::Puffed => PUFFED_SPEED,
        };
        let speed = base_speed * (0.75 + Math::random() * 0.5);
        let angle = Math::random() * TAU;
        self.movement.x = angle.cos() * speed;
        self.movement.y = angle.sin() * speed;
    }

    pub fn startle(&mut self) {
        match self.state {
            State::Normal => {
                self.state = State::Startled;
                self.cooldown = STARTLE_DURATION_S;
                self.reset_movement();
            }
            State::Startled if STARTLE_DURATION_S - self.cooldown > 0.5 => {
                self.cooldown = STARTLE_DURATION_S;
                self.reset_movement();
            }
            _ => {}
        }
    }

    pub fn puff(&mut self) {
        if matches!(self.state, State::Normal | State::Startled) {
            self.state = State::Puffed;
            self.reset_movement();
        }
        self.cooldown = PUFF_DURATION_S;
    }

    pub fn render(&self, canvas: &Canvas) {
        canvas.set_font(FONT_SIZE, "bold");

        match self.state {
            State::Normal | State::Startled => self.render_unpuffed(canvas),
            State::Puffed => self.render_puffed(canvas),
        }
    }

    fn render_unpuffed(&self, canvas: &Canvas) {
        let cw = char_width();
        let body_x = self.position.x + cw;
        let right_x = body_x + NORMAL_BODY_CHARS as f64 * cw;

        let baseline = self.position.y + FONT_SIZE;
        let fin_dy = FIN_OFFSETS[self.fin_offset];

        let body = match self.state {
            State::Normal => NORMAL_BODY,
            State::Startled => STARTLED_BODY,
            State::Puffed => unimplemented!(),
        };

        let (left_y, right_y) = if self.movement.x >= 0.0 {
            (baseline + fin_dy, baseline)
        } else {
            (baseline, baseline + fin_dy)
        };

        canvas.set_color(palette::PUFFER);
        canvas.fill_text("<", Vec2::new(self.position.x, left_y));
        canvas.fill_text(body, Vec2::new(body_x, baseline));
        canvas.fill_text(">", Vec2::new(right_x, right_y));
    }

    fn render_puffed(&self, canvas: &Canvas) {
        let spike_scale = (self.cooldown / PUFF_DURATION_S).max(0.1);
        let gap = FONT_SIZE * 0.3;
        let spike_height = FONT_SIZE * spike_scale;

        let top_baseline = self.position.y + FONT_SIZE;
        let mid_baseline = top_baseline + gap + FONT_SIZE;
        let bot_baseline = mid_baseline + gap + spike_height;

        canvas.set_color(palette::PUFFER);
        canvas.fill_text(PUFFED_MID, Vec2::new(self.position.x, mid_baseline));

        canvas.with_save(|canvas| {
            canvas.scale(1.0, spike_scale);
            canvas.set_color(palette::SPIKES);
            // scale() is anchored at origin, so divide y by spike_scale to get
            // the pre-scale coordinate that maps to the desired screen position.
            let pos_top = Vec2::new(self.position.x, top_baseline / spike_scale);
            let pos_bot = Vec2::new(self.position.x, bot_baseline / spike_scale);
            canvas.fill_text(PUFFED_TOP, pos_top);
            canvas.fill_text(PUFFED_BOT, pos_bot);
        });
    }

    pub fn bounds(&self) -> Rect {
        Rect {
            min: self.position,
            max: self.position + self.dimensions(),
        }
    }

    pub fn collide(&mut self, other: &Entity) {
        if matches!(other, Entity::Puffer(_)) {
            self.reset_movement();
        }
    }

    /// Puffer dimensions in pixels.
    fn dimensions(&self) -> Vec2 {
        match self.state {
            State::Normal | State::Startled => {
                let w = NORMAL_WIDTH_CHARS as f64 * char_width();
                let h = FONT_SIZE;
                Vec2::new(w, h)
            }
            State::Puffed => {
                let w = PUFFED_WIDTH_CHARS as f64 * char_width();
                let h = 3. * FONT_SIZE + 2. * FONT_SIZE * 0.3;
                Vec2::new(w, h)
            }
        }
    }
}

fn char_width() -> f64 {
    canvas::char_width(FONT_SIZE)
}
