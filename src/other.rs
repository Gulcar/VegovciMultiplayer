use macroquad::prelude::*;
use std::cell::Cell;
use std::collections::VecDeque;
use crate::{physics, AABB, StaticenAABBRef};
use crate::{LAYER_MAP, LAYER_PLAYER, LAYER_SWORD};

/// v bistvu polovicen width
pub fn screen_units_width() -> f32 {
    screen_units_height() * screen_width() / screen_height()
}

/// v bistvu polovicen height
pub fn screen_units_height() -> f32 {
    128.0
}

thread_local! {
    pub static KAMERA_POS: Cell<Vec2> = Cell::new(Vec2::ZERO);
    pub static SHOW_COLLIDERS: Cell<bool> = Cell::new(false);
}

pub fn posodobi_kamero() {
    let pixels_x = screen_units_width();
    let pixels_y = screen_units_height();

    set_camera(&Camera2D {
        target: KAMERA_POS.get(),
        zoom: vec2(1.0 / pixels_x, 1.0 / pixels_y),
        ..Default::default()
    });
}

pub fn pozicija_miske_v_svetu() -> Vec2 {
    let pos = mouse_position();
    let norm_pos = vec2(pos.0 / screen_width(), pos.1 / screen_height());
    let norm_from_center = norm_pos - vec2(0.5, 0.5);

    let pozicija = Vec2::new(
        norm_from_center.x * screen_units_width() * 2.0,
        norm_from_center.y * screen_units_height() * 2.0
    );

    pozicija + KAMERA_POS.get()
}

pub async fn load_texture_nearest(file: &str) -> Result<Texture2D, macroquad::Error> {
    let texture = load_texture(file).await?;
    texture.set_filter(FilterMode::Nearest);
    Ok(texture)
}

pub fn texture_params_source(x: f32, y: f32, w: f32, h: f32) -> DrawTextureParams {
    DrawTextureParams {
        source: Some(Rect::new(x, y, w, h)),
        ..Default::default()
    }
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}

pub fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    Color::new(
        a.r * (1.0 - t) + b.r * t,
        a.g * (1.0 - t) + b.g * t,
        a.b * (1.0 - t) + b.b * t,
        a.a * (1.0 - t) + b.a * t
    )
}

pub fn generate_map_colliders(map_image: Image, offset: Vec2) -> Vec<StaticenAABBRef> {
    let mut colliders = Vec::new();

    let mut obiskano = Vec::new();
    obiskano.resize(map_image.width() / 16 * map_image.height() / 16, false);

    for y in (0..map_image.height()).step_by(16).rev() {
        for x in (0..map_image.width()).step_by(16) {
            if map_image.get_pixel(x as u32, y as u32).a > 0.0 {

                if obiskano[x / 16 + (y / 16) * map_image.width() / 16] {
                    continue;
                }

                let mut extend_x = 0;
                let mut ex = x + 16;
                while ex < map_image.width() && map_image.get_pixel(ex as u32, y as u32).a > 0.0 {
                    extend_x += 1;
                    obiskano[ex / 16 + (y / 16) * map_image.width() / 16] = true;
                    ex += 16;
                }

                let aabb = AABB::new(
                    x as f32 + offset.x,
                    y as f32 + offset.y,
                    16.0 + (extend_x as f32) * 16.0,
                    16.0
                );
                colliders.push(physics::dodaj_staticen_obj(aabb, LAYER_MAP, LAYER_MAP | LAYER_PLAYER | LAYER_SWORD));
            }
        }
    }

    colliders
}

static mut MESSAGES: VecDeque<PopUpMessage> = VecDeque::new();

struct PopUpMessage {
    time: f32,
    total_time: f32,
    msg: String,
}

pub fn pop_up_msg(msg: String) {
    pop_up_msg_time(msg, 5.0);
}

pub fn pop_up_msg_time(msg: String, time: f32) {
    let messages = unsafe { &mut MESSAGES };
    messages.push_front(PopUpMessage { time, total_time: time, msg });
}

pub fn narisi_pop_up_messages(delta: f32) {
    let messages = unsafe { &mut MESSAGES };

    let mut y = screen_units_height() - 5.0;

    let mut i: i32 = 0;
    while (i as usize) < messages.len() {
        let msg = &mut messages[i as usize];

        let mut alpha = 1.0;
        if msg.total_time - msg.time < 0.1 {
            alpha = (msg.total_time - msg.time) * 10.0;
        }
        else if msg.time < 0.1 {
            alpha = msg.time * 10.0;
        }

        let pos = vec2(-screen_units_width() + 3.0, y) + KAMERA_POS.get();
        draw_text_ex(&msg.msg, pos.x, pos.y, TextParams {
            font_size: 32,
            font_scale: 0.35,
            color: Color::new(1.0, 1.0, 1.0, alpha),
            ..Default::default()
        });

        y -= 11.0;

        msg.time -= delta;
        if msg.time < 0.0 {
            messages.remove(i as usize);
            i -= 1;
        }

        i += 1;
    }
}
