use std::f32::consts::PI;

use macroquad::prelude::*;

use crate::{texture_params_source, DinamicenAABBRef, physics, AABB, pozicija_miske_v_svetu, KAMERA_POS, lerp, SHOW_COLLIDERS};
use crate::{LAYER_MAP, LAYER_PLAYER, LAYER_SWORD};

const PLAYER_SPEED: f32 = 75.0;
const JUMP_VEL: f32 = 500.0;
const MAX_VEL: f32 = 600.0;
const GRAVITY: f32 = 1500.0;

pub struct Animacija {
    pub cas: f32,
    cas_na_frame: f32,
    stevilo_framov: u32,
    prvi_frame: Rect,
    loop_anim: bool,
}

impl Animacija {
    pub fn new(prvi_frame: Rect, stevilo_framov: u32, cas_na_frame: f32, loop_anim: bool) -> Animacija {
        Animacija {
            cas: 0.0,
            cas_na_frame,
            stevilo_framov,
            prvi_frame,
            loop_anim
        }
    }

    pub fn posodobi(&mut self, delta: f32) {
        self.cas += delta;
        if self.loop_anim && self.cas >= self.cas_na_frame * self.stevilo_framov as f32 {
            self.cas -= self.cas_na_frame * self.stevilo_framov as f32;
        }
    }

    pub fn naredi_source_params(&self) -> DrawTextureParams {
        let xy = self.izr_frame_xy();
        texture_params_source(
            xy.x, xy.y,
            self.prvi_frame.w, self.prvi_frame.h
        )
    }

    pub fn izr_frame_xy(&self) -> Vec2 {
        Vec2::new(
            self.prvi_frame.x + (self.cas / self.cas_na_frame).floor() * self.prvi_frame.w,
            self.prvi_frame.y
        )
    }
}

pub struct Player {
    pub position: Vec2,
    pub rotation: f32,
    pub ime: String,

    pub health: i32,

    velocity_y: f32,
    jumps_allowed: i32,
    pub attack_time: f32,

    pub texture: Texture2D,
    aabb_ref: DinamicenAABBRef,

    sword_ref: DinamicenAABBRef,
    pub razdalja_meca: f32,

    pub animacije: Vec<Animacija>,
    pub trenutna_anim: usize,
}

impl Player {
    pub fn new(ime: String, position: Vec2, texture: Texture2D) -> Player {
        Player {
            position,
            rotation: 0.0,
            ime,
            health: 100,
            velocity_y: 0.0,
            jumps_allowed: 0,
            attack_time: 99.0,
            texture,
            aabb_ref: physics::dodaj_dinamicen_obj(AABB::from_vec(position, vec2(16.0, 28.0)), LAYER_PLAYER, LAYER_PLAYER | LAYER_MAP, 0),
            sword_ref: physics::dodaj_dinamicen_obj(AABB::from_vec(position, vec2(10.0, 10.0)), LAYER_SWORD, LAYER_SWORD | LAYER_MAP, 10),
            razdalja_meca: 0.0,
            animacije: vec![
                Animacija::new(Rect::new(0.0, 32.0, 32.0, 32.0), 2, 0.350, true), // idle 0
                Animacija::new(Rect::new(0.0, 64.0, 32.0, 32.0), 4, 0.100, true), // walk 1
            ],
            trenutna_anim: 0,
        }
    }

    pub fn posodobi(&mut self, delta: f32) {
        if self.health <= 0 {
            return;
        }

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

        let smer_meca = pozicija_miske_v_svetu() - (self.position + vec2(8.0, 12.0));
        let zeljena_pozicija = smer_meca.clamp_length_max(26.0) + self.position + vec2(3.0, 7.0);
        let pozicija_meca = physics::pozicija_obj(&self.sword_ref);
        let premik_meca = zeljena_pozicija - pozicija_meca;
        physics::premakni_obj(&self.sword_ref, premik_meca * 10.0 * delta);

        let dejanska_smer_meca = (pozicija_meca + vec2(5.0, 5.0)) - (self.position + vec2(8.0, 12.0));
        self.rotation = f32::atan2(dejanska_smer_meca.y, dejanska_smer_meca.x);
        self.razdalja_meca = dejanska_smer_meca.length() - 3.0;

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
        if self.health <= 0 {
            return;
        }
        let position = physics::pozicija_obj(&self.aabb_ref);
        Player::narisi_iz(&self.texture, position, self.get_anim().izr_frame_xy(), self.rotation, self.razdalja_meca, self.attack_time, &self.ime, self.health);
    }

    pub fn narisi_iz(tekstura: &Texture2D, position: Vec2, anim_frame_xy: Vec2, rotacija: f32, razdalja_meca: f32, attack_time: f32, ime: &str, health: i32) {
        let draw_position = position - vec2(8.0, 4.0);
        let mut params = texture_params_source(anim_frame_xy.x, anim_frame_xy.y, 32.0, 32.0);
        params.flip_x = rotacija > PI / 2.0 || rotacija < -PI / 2.0;
        draw_texture_ex(tekstura, draw_position.x, draw_position.y, WHITE, params);

        let attack_amount = -f32::powi(attack_time / 0.3 - 1.0, 3);
        let sword_dist = razdalja_meca + 22.0 * attack_amount.max(0.0);

        let center = draw_position + vec2(16.0, 16.0);
        let sword_offset = Vec2::from_angle(rotacija) * sword_dist;
        let sword_draw_position = center + sword_offset - vec2(8.0, 8.0);
        let mut params = texture_params_source(64.0, 16.0, 16.0, 16.0);
        params.rotation = f32::atan2(sword_offset.y, sword_offset.x) + PI / 4.0;
        draw_texture_ex(tekstura, sword_draw_position.x, sword_draw_position.y, WHITE, params);

        if SHOW_COLLIDERS.get() && attack_time == 0.0 {
            let hitbox = Player::calc_sword_hitbox(position, attack_time, razdalja_meca, rotacija);
            draw_rectangle(hitbox.x, hitbox.y, hitbox.w, hitbox.h, RED);
        }

        let text_params = TextParams { font_size: 32, font_scale: 0.25, ..Default::default() };
        let dimensions = measure_text(ime, None, text_params.font_size, text_params.font_scale);
        draw_text_ex(ime, draw_position.x + 16.0 - dimensions.width / 2.0, position.y - 5.0, text_params);

        if health != -1 {
            const WIDTH: f32 = 25.0;
            draw_rectangle(center.x - WIDTH / 2.0, position.y - 3.6, WIDTH, 3.0, BLACK);
            let fill = WIDTH * health as f32 / 100.0;
            draw_rectangle(center.x - WIDTH / 2.0, position.y - 3.6, fill, 3.0, GREEN);
        }
    }

    pub fn get_anim(&self) -> &Animacija {
        &self.animacije[self.trenutna_anim]
    }

    pub fn calc_sword_hitbox(player_pos: Vec2, attack_time: f32, razdalja_meca: f32, rotacija: f32) -> AABB {
        let center = player_pos - vec2(8.0, 4.0) + vec2(16.0, 16.0);

        let attack_amount = -f32::powi(attack_time / 0.3 - 1.0, 3);
        let sword_dist = razdalja_meca + 22.0 * attack_amount.max(0.0);
        let sword_offset = Vec2::from_angle(rotacija) * sword_dist;

        let pos = center + sword_offset / 1.5 - vec2(16.0, 16.0);
        AABB::from_vec(pos, vec2(32.0, 32.0))
    }

    pub fn nastavi_pozicijo(&mut self, position: Vec2) {
        self.position = position;
        physics::premakni_obj_na(&self.aabb_ref, position);
        physics::premakni_obj_na(&self.sword_ref, position);
    }
}

