use std::f32::consts::PI;

use macroquad::prelude::*;

use crate::{texture_params_source, DinamicenAABBRef, physics, AABB, pozicija_miske_v_svetu, KAMERA_POS, lerp};

const PLAYER_SPEED: f32 = 75.0;
const JUMP_VEL: f32 = 500.0;
const MAX_VEL: f32 = 600.0;
const GRAVITY: f32 = 1500.0;

pub struct Player {
    pub position: Vec2,
    velocity_y: f32,
    jumps_allowed: i32,
    texture: Texture2D,
    aabb_ref: DinamicenAABBRef,
}

impl Player {
    pub fn new(position: Vec2, texture: Texture2D) -> Player {
        Player {
            position,
            velocity_y: 0.0,
            jumps_allowed: 0,
            texture,
            aabb_ref: physics::dodaj_dinamicen_obj(AABB::from_vec(position, vec2(16.0, 28.0))),
        }
    }

    pub fn posodobi(&mut self, delta: f32) {
        let nova_pozicija = physics::pozicija_obj(&self.aabb_ref);
        if nova_pozicija.y == self.position.y {
            self.velocity_y = 0.0;
            self.jumps_allowed = 2;
        }
        self.position = nova_pozicija;

        let mut premik = Vec2::ZERO;

        if is_key_down(KeyCode::A) {
            premik.x -= PLAYER_SPEED * delta;
        }
        if is_key_down(KeyCode::D) {
            premik.x += PLAYER_SPEED * delta;
        }

        if self.jumps_allowed > 0 && is_key_pressed(KeyCode::W) {
            self.velocity_y = -JUMP_VEL;
            self.jumps_allowed -= 1;
        }
        self.velocity_y += GRAVITY * delta;
        if self.velocity_y.abs() > MAX_VEL {
            self.velocity_y = self.velocity_y.signum() * MAX_VEL;
        }

        premik.y += self.velocity_y * delta;

        physics::premakni_obj(&self.aabb_ref, premik);

        let zeljena_pozicija_kamere = Vec2::lerp(self.position, pozicija_miske_v_svetu(), 0.1);
        let pozicija_kamere = KAMERA_POS.get();
        // zelim pocasnejse premikanje kamere na y
        let nova_pozicija = vec2(
            lerp(pozicija_kamere.x, zeljena_pozicija_kamere.x, 10.0 * delta),
            lerp(pozicija_kamere.y, zeljena_pozicija_kamere.y, 3.0 * delta)
        );
        KAMERA_POS.set(nova_pozicija);
    }

    pub fn narisi(&self) {
        let position = physics::pozicija_obj(&self.aabb_ref);
        let draw_position = position - vec2(8.0, 4.0);
        let params = texture_params_source(0.0, 0.0, 32.0, 32.0);
        draw_texture_ex(&self.texture, draw_position.x, draw_position.y, WHITE, params);

        let center = draw_position + vec2(16.0, 16.0);
        let mouse_position = pozicija_miske_v_svetu();
        let sword_offset = (mouse_position - center).normalize() * 24.0;
        let sword_draw_position = center + sword_offset - vec2(8.0, 8.0);
        let mut params = texture_params_source(64.0, 16.0, 16.0, 16.0);
        params.rotation = f32::atan2(sword_offset.y, sword_offset.x) + PI / 4.0;
        draw_texture_ex(&self.texture, sword_draw_position.x, sword_draw_position.y, WHITE, params);
    }
}

