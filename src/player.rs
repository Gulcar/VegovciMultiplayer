use std::f32::consts::PI;

use macroquad::prelude::*;

use crate::{texture_params_source, DinamicenAABBRef, physics, AABB, pozicija_miske_v_svetu, KAMERA_POS, lerp};

const PLAYER_SPEED: f32 = 75.0;
const JUMP_VEL: f32 = 500.0;
const MAX_VEL: f32 = 600.0;
const GRAVITY: f32 = 1500.0;

struct Animacija {
    cas: f32,
    cas_na_frame: f32,
    stevilo_framov: u32,
    prvi_frame: Rect,
    loop_anim: bool,
}

impl Animacija {
    fn new(prvi_frame: Rect, stevilo_framov: u32, cas_na_frame: f32, loop_anim: bool) -> Animacija {
        Animacija {
            cas: 0.0,
            cas_na_frame,
            stevilo_framov,
            prvi_frame,
            loop_anim
        }
    }

    fn posodobi(&mut self, delta: f32) {
        self.cas += delta;
        if self.loop_anim && self.cas >= self.cas_na_frame * self.stevilo_framov as f32 {
            self.cas -= self.cas_na_frame * self.stevilo_framov as f32;
        }
    }

    fn naredi_source_params(&self) -> DrawTextureParams {
        texture_params_source(
            self.prvi_frame.x + (self.cas / self.cas_na_frame).floor() * self.prvi_frame.w,
            self.prvi_frame.y,
            self.prvi_frame.w, self.prvi_frame.h
        )
    }
}

pub struct Player {
    pub position: Vec2,
    pub ime: String,

    velocity_y: f32,
    jumps_allowed: i32,
    attack_time: f32,

    texture: Texture2D,
    aabb_ref: DinamicenAABBRef,

    animacije: Vec<Animacija>,
    trenutna_anim: usize,
}

impl Player {
    pub fn new(ime: String, position: Vec2, texture: Texture2D) -> Player {
        Player {
            position,
            ime,
            velocity_y: 0.0,
            jumps_allowed: 0,
            attack_time: 99.0,
            texture,
            aabb_ref: physics::dodaj_dinamicen_obj(AABB::from_vec(position, vec2(16.0, 28.0))),
            animacije: vec![
                Animacija::new(Rect::new(0.0, 32.0, 32.0, 32.0), 2, 0.350, true),
                Animacija::new(Rect::new(0.0, 64.0, 32.0, 32.0), 4, 0.100, true),
            ],
            trenutna_anim: 0,
        }
    }

    pub fn posodobi(&mut self, delta: f32) {
        let nova_pozicija = physics::pozicija_obj(&self.aabb_ref);
        let mut is_grounded = false;
        if (nova_pozicija.y - self.position.y).abs() < 0.00001 {
            self.velocity_y = 0.0;
            self.jumps_allowed = 1;
            is_grounded = true;
        }
        self.position = nova_pozicija;

        let mut premik = Vec2::ZERO;

        if is_key_down(KeyCode::A) {
            premik.x -= PLAYER_SPEED * delta;
        }
        if is_key_down(KeyCode::D) {
            premik.x += PLAYER_SPEED * delta;
        }

        if (self.jumps_allowed > 0 || is_grounded) && is_key_pressed(KeyCode::W) {
            self.velocity_y = -JUMP_VEL;
            if is_grounded == false {
                self.jumps_allowed -= 1;
            }
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

        if is_mouse_button_pressed(MouseButton::Left) {
            self.attack_time = 0.0;
        } else {
            self.attack_time += delta;
        }

        self.animacije[self.trenutna_anim].posodobi(delta);
        if premik.x != 0.0 {
            self.trenutna_anim = 1;
        } else {
            self.trenutna_anim = 0;
        }
    }

    pub fn narisi(&self) {
        let position = physics::pozicija_obj(&self.aabb_ref);
        let draw_position = position - vec2(8.0, 4.0);
        let mut params = self.animacije[self.trenutna_anim].naredi_source_params();
        params.flip_x = pozicija_miske_v_svetu().x < draw_position.x + 16.0;
        draw_texture_ex(&self.texture, draw_position.x, draw_position.y, WHITE, params);

        let attack_amount = -f32::powi(self.attack_time / 0.3 - 1.0, 3);
        let sword_dist = 18.0 + 22.0 * attack_amount.max(0.0);

        let center = draw_position + vec2(16.0, 16.0);
        let mouse_position = pozicija_miske_v_svetu();
        let sword_offset = (mouse_position - center).normalize() * sword_dist;
        let sword_draw_position = center + sword_offset - vec2(8.0, 8.0);
        let mut params = texture_params_source(64.0, 16.0, 16.0, 16.0);
        params.rotation = f32::atan2(sword_offset.y, sword_offset.x) + PI / 4.0;
        draw_texture_ex(&self.texture, sword_draw_position.x, sword_draw_position.y, WHITE, params);

        let text_params = TextParams { font_size: 32, font_scale: 0.25, ..Default::default() };
        let dimensions = measure_text(&self.ime, None, text_params.font_size, text_params.font_scale);
        draw_text_ex(&self.ime, draw_position.x + 16.0 - dimensions.width / 2.0, position.y - 5.0, text_params);
    }

    pub fn get_rotation(&self) -> f32 {
        let to = pozicija_miske_v_svetu() - (self.position + vec2(8.0, 12.0));
        f32::atan2(to.y, to.x)
    }
}

