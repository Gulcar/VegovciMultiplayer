use std::{f32::consts::PI, ops::Range};

use macroquad::prelude::*;

use crate::{lerp_color, lerp};

#[derive(Default)]
struct ParticlesState {
    batches: Vec<Batch>,
}

struct Batch {
    particles: Vec<Particle>,
    time: f32,
    lifetime: f32,
}

#[derive(Clone, Default)]
pub struct ParticleParams {
    pub amount: Range<usize>,
    pub lifetime: f32,
    pub speed_start: Range<f32>,
    pub speed_end: Range<f32>,
    pub color_start: Range<Color>,
    pub color_end: Range<Color>,
    pub size_start: Range<f32>,
    pub size_end: Range<f32>,
}

struct Particle {
    position: Vec2,
    direction: Vec2,
    speed_start: f32,
    speed_end: f32,
    color_start: Color,
    color_end: Color,
    size_start: f32,
    size_end: f32,
}

static mut STATE: Option<ParticlesState> = None;

pub fn init() {
    unsafe { STATE = Some(ParticlesState::default()); }
}

pub fn spawn(position: Vec2, direction: Option<Vec2>, p: &ParticleParams) {
    let state = unsafe { STATE.as_mut().unwrap() };

    let amount = rand::gen_range(p.amount.start, p.amount.end - 1);

    let mut batch = Batch {
        particles: Vec::with_capacity(amount),
        time: 0.0,
        lifetime: p.lifetime,
    };

    for _ in 0..amount {

        let mut dir = Vec2::from_angle(rand::gen_range(0.0, 2.0 * PI));
        if let Some(d) = direction {
            dir += d;
            dir = dir.normalize();
        }

        batch.particles.push(Particle {
            position,
            direction: dir,
            speed_start: rand::gen_range(p.speed_start.start, p.speed_start.end),
            speed_end: rand::gen_range(p.speed_end.start, p.speed_end.end),
            color_start: lerp_color(p.color_start.start, p.color_start.end, rand::gen_range(0.0, 1.0)),
            color_end: lerp_color(p.color_end.start, p.color_end.end, rand::gen_range(0.0, 1.0)),
            size_start: rand::gen_range(p.size_start.start, p.size_start.end),
            size_end: rand::gen_range(p.size_end.start, p.size_end.end),
        });
    }

    state.batches.push(batch);
}

pub fn narisi(delta: f32) {
    let state = unsafe { STATE.as_mut().unwrap() };

    let mut to_delete = None;

    for (i, batch) in state.batches.iter_mut().enumerate() {

        batch.time += delta;
        if batch.time > batch.lifetime {
            to_delete = Some(i);
            continue;
        }
        
        let t = batch.time / batch.lifetime;

        for particle in &mut batch.particles {
            let speed = lerp(particle.speed_start, particle.speed_end, t);
            let color = lerp_color(particle.color_start, particle.color_end, t);
            let size = lerp(particle.size_start, particle.size_end, t);
            let velocity = particle.direction * speed;
            particle.position += velocity * delta;
            draw_rectangle(
                particle.position.x - size / 2.0,
                particle.position.y - size / 2.0,
                size, size, color
            );
        }
    }

    if let Some(i) = to_delete {
        state.batches.swap_remove(i);
    }
}
