use macroquad::prelude::*;

use crate::{texture_params_source, DinamicenAABBRef, physics, AABB};

const PLAYER_SPEED: f32 = 75.0;
const JUMP_VEL: f32 = 500.0;
const MAX_VEL: f32 = 600.0;
const GRAVITY: f32 = 1500.0;

pub struct Player {
    pub position: Vec2,
    velocity_y: f32,
    texture: Texture2D,
    aabb_ref: DinamicenAABBRef,
}

impl Player {
    pub fn new(position: Vec2, texture: Texture2D) -> Player {
        Player {
            position,
            velocity_y: 0.0,
            texture,
            aabb_ref: physics::dodaj_dinamicen_obj(AABB::from_vec(position, vec2(16.0, 28.0))),
        }
    }

    pub fn posodobi(&mut self, delta: f32) {
        let nova_pozicija = physics::pozicija_obj(&self.aabb_ref);
        if nova_pozicija.y == self.position.y {
            self.velocity_y = 0.0;
        }
        self.position = nova_pozicija;

        let mut premik = Vec2::ZERO;

        if is_key_down(KeyCode::A) {
            premik.x -= PLAYER_SPEED * delta;
        }
        if is_key_down(KeyCode::D) {
            premik.x += PLAYER_SPEED * delta;
        }

        if is_key_pressed(KeyCode::W) {
            self.velocity_y = -JUMP_VEL;
        }
        self.velocity_y += GRAVITY * delta;
        if self.velocity_y.abs() > MAX_VEL {
            self.velocity_y = self.velocity_y.signum() * MAX_VEL;
        }

        premik.y += self.velocity_y * delta;

        physics::premakni_obj(&self.aabb_ref, premik);
    }

    pub fn narisi(&self) {
        let position = physics::pozicija_obj(&self.aabb_ref);
        let params = texture_params_source(0.0, 0.0, 32.0, 32.0);
        draw_texture_ex(&self.texture, position.x - 8.0, position.y - 4.0, WHITE, params);
    }
}

