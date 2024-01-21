use macroquad::prelude::*;

mod player;
use player::*;
mod collision;
use collision::*;

fn posodobi_kamero() {
    let width = screen_width();
    let height = screen_height();

    let pixels = 128.0;

    set_camera(&Camera2D {
        target: vec2(0.0, 0.0),
        zoom: vec2((1.0 / pixels) / width * height, 1.0 / pixels),
        ..Default::default()
    });
}

async fn load_texture_nearest(file: &str) -> Result<Texture2D, macroquad::Error> {
    let texture = load_texture(file).await?;
    texture.set_filter(FilterMode::Nearest);
    Ok(texture)
}

fn texture_params_source(x: f32, y: f32, w: f32, h: f32) -> DrawTextureParams {
    DrawTextureParams {
        source: Some(Rect::new(x, y, w, h)),
        ..Default::default()
    }
}

#[macroquad::main("VegovciMultiplayer")]
async fn main() {
    println!("pozdravljen svet!");

    let vegovec_texture = load_texture_nearest("assets/vegovec.png").await.unwrap();
    let map_texture = load_texture_nearest("assets/map.png").await.unwrap();

    physics::init();

    let mut map_aabb_refs = Vec::new();
    map_aabb_refs.push(physics::dodaj_staticen_obj(AABB::new(-96.0, 96.0, 192.0, 32.0)));
    map_aabb_refs.push(physics::dodaj_staticen_obj(AABB::new(32.0, 64.0, 16.0, 32.0)));

    let test_aabb = physics::dodaj_dinamicen_obj(AABB::new(-32.0, 64.0, 16.0, 32.0));

    let mut player = Player::new(vec2(0.0, 64.0), vegovec_texture);

    loop {
        let delta = get_frame_time().min(1.0 / 15.0);
        player.posodobi(delta);

        physics::resi_trke();
        physics::resi_trke();

        posodobi_kamero();
        clear_background(Color::new(0.1, 0.1, 0.1, 1.0));

        draw_texture(&map_texture, -128.0, 0.0, WHITE);
        player.narisi();

        physics::narisi_aabbje();
        
        next_frame().await;
    }
}

