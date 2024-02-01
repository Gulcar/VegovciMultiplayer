use macroquad::prelude::*;
use std::env;

mod player;
mod collision;
mod network;
mod particles;
mod other;

use player::*;
use collision::*;
use network::*;
use other::*;

fn print_usage_exit(first_arg: &str) -> ! {
    eprintln!("ERROR incorect parameters!");
    eprintln!("usage: {} <user_name> <server_ip>", first_arg);
    eprintln!("or   : {} <user_name> host", first_arg);
    std::process::exit(1)
}

#[macroquad::main("VegovciMultiplayer")]
async fn main() {
    println!("pozdravljen svet!");

    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        print_usage_exit(&args[0]);
    }
    let user_name = &args[1];
    let server_ip = &args[2];
    let is_host = args[2] == "host";

    // dodatni argumenti
    for i in 3..args.len() {
        match args[i].as_str() {
            "--colliders" => SHOW_COLLIDERS.set(true),
            _ => panic!("unknown option: {}", args[i])
        }
    }

    if is_host {
        println!("v nacinu streznika!");
    }

    let mut net_interface = {
        if is_host { NetInterface::Server(Server::new(user_name.clone())) }
        else { NetInterface::Client(Client::new(&server_ip, user_name.clone())) }
    };

    let vegovec_texture = load_texture_nearest("assets/vegovec.png").await.unwrap();
    let map_texture = load_texture_nearest("assets/map.png").await.unwrap();

    physics::init();
    particles::init();

    let _map_aabb_refs = generate_map_colliders(map_texture.get_texture_data(), vec2(-256.0, -128.0));
    //map_aabb_refs.push(physics::dodaj_staticen_obj(AABB::new(-96.0, 48.0, 192.0, 32.0)));
    //map_aabb_refs.push(physics::dodaj_staticen_obj(AABB::new(32.0, 16.0, 16.0, 32.0)));

    //let _test_aabb = physics::dodaj_dinamicen_obj(AABB::new(-32.0, 16.0, 16.0, 32.0), LAYER_MAP, LAYER_MAP | LAYER_PLAYER | LAYER_SWORD, 0);

    let mut player = Player::new(user_name.clone(), vec2(0.0, 0.0), vegovec_texture);

    println!("stevilo staticnih objektov: {}", physics::st_staticnih_obj());
    println!("stevilo dinamicnih objektov: {}", physics::st_dinamicnih_obj());

    loop {
        let delta = get_frame_time().min(1.0 / 15.0);

        match net_interface {
            NetInterface::Server(ref mut server) => {
                server.listen();
                server.recv();
                server.posodobi(delta, &mut player);
                player.health = server.health;
                if player.attack_time == 0.0 {
                    server.attack_host(&player);
                }
                server.poslji_vse_state(&player);
            },
            NetInterface::Client(ref mut client) => {
                client.recv(&mut player);
                player.health = client.health;
                let state = State {
                    id: client.id,
                    position: (player.position.x, player.position.y),
                    rotation: player.rotation,
                    anim_frame: player.animacije[player.trenutna_anim].izr_frame_xy().into(),
                    attack_time: player.attack_time,
                    razdalja_meca: player.razdalja_meca,
                };
                let send_buf = bincode::serialize(&Message::PlayerState(state)).unwrap();
                client.send(&send_buf);
            },
        }

        player.posodobi(delta);

        physics::resi_trke();
        physics::resi_trke();

        posodobi_kamero();
        clear_background(Color::new(0.1, 0.1, 0.1, 1.0));

        draw_texture(&map_texture, -256.0, -128.0, WHITE);
        player.narisi();

        particles::narisi(delta);

        if SHOW_COLLIDERS.get() {
            physics::narisi_aabbje();
        }

        match net_interface {
            NetInterface::Server(ref server) => {
                server.narisi_cliente(&player.texture);
            }
            NetInterface::Client(ref client) => {
                client.narisi_cliente(&player.texture);
            }
        }

        let pos = vec2(-screen_units_width() + 3.0, -screen_units_height() + 11.0) + KAMERA_POS.get();
        draw_text_ex(&format!("{} fps", get_fps()), pos.x, pos.y, TextParams {
            font_size: 32,
            font_scale: 0.35,
            ..Default::default()
        });

        next_frame().await;
    }
}

