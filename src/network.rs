use std::{net::{TcpStream, TcpListener, SocketAddr}, io::{ErrorKind, Write, BufReader}, collections::HashMap};
use macroquad::prelude::*;
use serde::{Serialize, Deserialize};

use crate::{Player, DinamicenAABBRef, physics, LAYER_PLAYER, particles, HIT_PARTICLES};
use crate::AABB;

const PORT: u16 = 5356;
const RESPAWN_TIME: f32 = 3.0;
const FALLOFF_Y: f32 = 500.0;

fn prepare_socket(stream: &mut TcpStream) {
    stream.set_nonblocking(true).unwrap();
    stream.set_nodelay(true).unwrap();
}

/// vrne true ce je disconnect
fn handle_bincode_error(e: Box<bincode::ErrorKind>) -> bool {
    match *e {
        bincode::ErrorKind::Io(e) => {
            match e.kind() {
                ErrorKind::WouldBlock => {
                },
                ErrorKind::UnexpectedEof | ErrorKind::ConnectionReset => {
                    return true; // disconnect
                },
                _ => {
                    eprintln!("io err recv: {:?}", e);
                }
            }
        },
        _ => {
            eprintln!("err recv: {:?}", e);
        }
    }

    return false;
}

pub struct ServerConnection {
    reader: BufReader<TcpStream>,
    state: State,
    addr: SocketAddr,
    user_name: String,
    aabb_ref: DinamicenAABBRef,
    health: i32,
    respawn_timer: f32,
}

pub struct Server {
    listener: TcpListener,
    clients: Vec<ServerConnection>,
    naslednji_id: u32,
    user_name: String,
    pub health: i32,
    respawn_timer: f32,
}

impl Server {
    pub fn new(user_name: String) -> Server {
        let listener = TcpListener::bind(("0.0.0.0", PORT)).unwrap();
        listener.set_nonblocking(true).unwrap();
        Server {
            listener,
            clients: Vec::new(),
            naslednji_id: 1,
            user_name,
            health: 100,
            respawn_timer: 0.0,
        }
    }

    fn on_start_conn(&self, stream: &mut TcpStream) {
        prepare_socket(stream);

        let mut send_buf: Vec<u8> = Vec::new();
        send_buf.append(&mut bincode::serialize(&Message::DodeljenId(self.naslednji_id)).unwrap());

        let msg = Message::UserName((0, self.user_name.clone()));
        send_buf.append(&mut bincode::serialize(&msg).unwrap());

        for client in &self.clients {
            let msg = Message::UserName((client.state.id, client.user_name.clone()));
            send_buf.append(&mut bincode::serialize(&msg).unwrap());
        }

        stream.write(&send_buf).unwrap();
    }

    pub fn listen(&mut self) {
        for conn_attempt in self.listener.incoming() {
            match conn_attempt {
                Ok(mut stream) => {
                    let addr = stream.peer_addr().unwrap();
                    println!("new client connected from {}", addr);
                    self.on_start_conn(&mut stream);
                    self.clients.push(ServerConnection {
                        reader: BufReader::new(stream),
                        state: State {
                            id: self.naslednji_id,
                            ..Default::default()
                        },
                        addr,
                        user_name: String::new(),
                        aabb_ref: physics::dodaj_dinamicen_obj(AABB::new(0.0, 0.0, 16.0, 28.0), LAYER_PLAYER, 0, self.naslednji_id),
                        health: 100,
                        respawn_timer: 0.0,
                    });
                    self.naslednji_id += 1;
                },
                Err(e) => {
                    if e.kind() == ErrorKind::WouldBlock {
                        break;
                    }
                    eprintln!("error in TcpListener incoming: {}", e);
                }
            }
        }
    }

    fn handle_attack(&mut self, hit_list: Vec<(u32, AABB)>, exclude_id: u32, pozicija: Vec2, smer: Vec2) {
        //println!("attack_hit {}: {:?}", hit_list.len(), hit_list);
        for (id, aabb) in &hit_list {
            if *id == exclude_id {
                continue;
            }

            let particles_pos = aabb.ray_hit(pozicija, smer);
            if particles_pos.is_none() {
                continue;
            }
            let particles_pos = particles_pos.unwrap();

            if *id == 0 {
                if self.health > 0 {
                    self.health -= 10;
                    if self.health <= 0 {
                        self.health = 0;
                        self.respawn_timer = RESPAWN_TIME;
                        println!("umrl id {}", *id);
                    }
                    particles::spawn(particles_pos, None, &HIT_PARTICLES);
                    self.send_msg_all(Message::HitParticles(particles_pos.into()));
                }
            }
            else if let Some(client) = self.clients.iter_mut().find(|c| c.state.id == *id && c.health > 0) {
                client.health -= 10;
                if client.health <= 0 {
                    client.health = 0;
                    client.respawn_timer = RESPAWN_TIME;
                    println!("umrl id {}", *id);
                }
                let msg = Message::Attack(client.health);
                Server::send_msg(client, msg);

                particles::spawn(particles_pos, None, &HIT_PARTICLES);
                self.send_msg_all(Message::HitParticles(particles_pos.into()));
            }
        }
    }

    pub fn attack_host(&mut self, player: &Player) {
        let hitbox = Player::calc_sword_hitbox(player.position, player.attack_time, player.razdalja_meca, player.rotation);
        let found = physics::area_query(hitbox, LAYER_PLAYER);

        let pozicija = player.position + vec2(8.0, 12.0);
        let smer = Vec2::from_angle(player.rotation);
        self.handle_attack(found, 0, pozicija, smer);
    }

    fn attack(&mut self, conn_i: usize) {
        let client = &self.clients[conn_i];
        let state = &client.state;

        let hitbox = Player::calc_sword_hitbox(state.position.into(), state.attack_time, state.razdalja_meca, state.rotation);
        let found = physics::area_query(hitbox, LAYER_PLAYER);

        let pozicija = Vec2::from(client.state.position) + vec2(8.0, 12.0);
        let smer = Vec2::from_angle(client.state.rotation);
        self.handle_attack(found, state.id, pozicija, smer);
    }

    fn handle_msg(&mut self, msg: Message, conn_i: usize) {
        let client = &mut self.clients[conn_i];
        match msg {
            Message::PlayerState(state) => {
                let id = client.state.id;
                client.state = state.clone();
                client.state.id = id;

                physics::premakni_obj_na(&client.aabb_ref, client.state.position.into());

                if state.attack_time == 0.0 {
                    self.attack(conn_i);
                }
            },
            Message::UserName((_id, name)) => {
                client.user_name = name.clone();
                let msg = Message::UserName((client.state.id, name));
                self.send_msg_all(msg);
            }
            _ => {},
        }
    }

    pub fn recv(&mut self) {
        let mut i: i32 = 0;
        while i < self.clients.len() as i32 {

            loop {
                match bincode::deserialize_from::<&mut BufReader<TcpStream>, Message>(&mut self.clients[i as usize].reader) {
                    Ok(msg) => {
                        self.handle_msg(msg, i as usize);
                        //println!("recv: {:?}", msg);
                    },
                    Err(e) => {
                        if handle_bincode_error(e) {
                            println!("client disconnected {:?}", self.clients[i as usize].addr);
                            self.clients.swap_remove(i as usize);
                            i -= 1;
                        }
                        break;
                    }
                }
            }

            i += 1;
        }
    }

    pub fn send_to_all(&mut self, bytes: &[u8]) {
        for conn in self.clients.iter_mut() {
            if let Err(e) = conn.reader.get_mut().write(bytes) {
                eprintln!("err socket write: {:?}", e);
            }
        }
    }

    pub fn send_msg_all(&mut self, msg: Message) {
        let send_buf = bincode::serialize(&msg).unwrap();
        self.send_to_all(&send_buf);
    }

    pub fn send_msg(conn: &mut ServerConnection, msg: Message) {
        let send_buf = bincode::serialize(&msg).unwrap();
        conn.reader.get_mut().write(&send_buf).unwrap();
    }

    pub fn narisi_cliente(&self, tekstura: &Texture2D) {
        for conn in &self.clients {
            if conn.health <= 0 { continue; }
            //draw_rectangle_lines(conn.state.position.0, conn.state.position.1, 16.0, 28.0, 1.0, macroquad::color::RED);
            let state = &conn.state;
            Player::narisi_iz(tekstura, state.position.into(), state.anim_frame.into(), state.rotation, state.razdalja_meca, state.attack_time, &conn.user_name, -1);
        }
    }

    pub fn poslji_vse_state(&mut self, player: &Player) {
        let mut states: Vec<State> = self.clients.iter()
            .filter(|c| c.health > 0)
            .map(|c| c.state.clone())
            .collect();
        if self.health > 0 {
            states.push(State {
                id: 0, // gazda/host id
                position: (player.position.x, player.position.y),
                rotation: player.rotation,
                anim_frame: player.get_anim().izr_frame_xy().into(),
                attack_time: player.attack_time,
                razdalja_meca: player.razdalja_meca,
            });
        }
        self.send_msg_all(Message::AllPlayersState(states));
    }

    fn get_respawn_location() -> Vec2 {
        Vec2::new(
            rand::gen_range(-200.0, 200.0),
            -250.0
        )
    }

    pub fn posodobi(&mut self, delta: f32, player: &mut Player) {
        if self.health <= 0 {
            self.respawn_timer -= delta;
            if self.respawn_timer <= 0.0 {
                self.health = 100;
                player.nastavi_pozicijo(Server::get_respawn_location());
            }
        }

        for client in &mut self.clients {
            if client.health <= 0 {
                client.respawn_timer -= delta;
                if client.respawn_timer <= 0.0 {
                    client.health = 100;
                    let msg = Message::Respawn(Server::get_respawn_location().into());
                    Server::send_msg(client, msg);
                }
            }
        }

        if self.health > 0 && player.position.y > FALLOFF_Y {
            self.health = 0;
            self.respawn_timer = RESPAWN_TIME;
        }

        for client in &mut self.clients {
            if client.health > 0 && client.state.position.1 > FALLOFF_Y {
                client.health = 0;
                client.respawn_timer = RESPAWN_TIME;
                let msg = Message::Attack(client.health);
                Server::send_msg(client, msg);
            }
        }
    }
}

pub struct Client {
    pub id: u32,
    reader: BufReader<TcpStream>,
    net_states: Vec<State>,
    net_user_names: HashMap<u32, String>,
    pub health: i32,
}

impl Client {
    pub fn new(addr: &str, name: String) -> Client {
        let mut stream = match TcpStream::connect((addr, PORT)) {
            Ok(s) => s,
            Err(e) => panic!("ERROR povezava neuspešna: {}", e),
        };
        prepare_socket(&mut stream);
        let msg = Message::UserName((u32::MAX, name));
        stream.write(&bincode::serialize(&msg).unwrap()).unwrap();
        Client {
            id: u32::MAX,
            reader: BufReader::new(stream),
            net_states: Vec::new(),
            net_user_names: HashMap::new(),
            health: 100,
        }
    }

    pub fn send(&mut self, bytes: &[u8]) {
        self.reader.get_mut().write(bytes).unwrap();
    }

    pub fn handle_msg(&mut self, msg: Message, player: &mut Player) {
        match msg {
            Message::DodeljenId(id) => {
                self.id = id;
                println!("dobil id: {}", id);
            },
            Message::AllPlayersState(states) => {
                self.net_states = states;
            },
            Message::UserName((id, name)) => {
                self.net_user_names.insert(id, name);
            }
            Message::Attack(new_health) => {
                self.health = new_health;
            }
            Message::Respawn((x, y)) => {
                self.health = 100;
                player.nastavi_pozicijo(Vec2::new(x, y));
            }
            Message::HitParticles((x, y)) => {
                particles::spawn((x, y).into(), None, &HIT_PARTICLES);
            }
            _ => {},
        }
    }

    pub fn recv(&mut self, player: &mut Player) {
        loop {
            match bincode::deserialize_from::<&mut BufReader<TcpStream>, Message>(&mut self.reader) {
                Ok(msg) => {
                    //println!("recv: {:?}", msg);
                    self.handle_msg(msg, player);
                },
                Err(e) => {
                    if handle_bincode_error(e) {
                        println!("disconnected from the server");
                        std::process::exit(0);
                    }
                    break;
                }
            }
        }
    }

    pub fn narisi_cliente(&self, tekstura: &Texture2D) {
        let default_name = "peer".to_string();

        for state in &self.net_states {
            if state.id == self.id {
                continue;
            }
            //draw_rectangle_lines(state.position.0, state.position.1, 16.0, 28.0, 1.0, macroquad::color::RED);
            let name = self.net_user_names.get(&state.id).unwrap_or(&default_name);
            Player::narisi_iz(tekstura, state.position.into(), state.anim_frame.into(), state.rotation, state.razdalja_meca, state.attack_time, name, -1);
        }
    }
}

pub enum NetInterface {
    Server(Server),
    Client(Client),
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct State {
    pub id: u32,
    pub position: (f32, f32),
    pub rotation: f32,
    pub anim_frame: (f32, f32),
    pub attack_time: f32,
    pub razdalja_meca: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    DodeljenId(u32),
    UserName((u32, String)),
    PlayerState(State),
    AllPlayersState(Vec<State>),
    Attack(i32),
    Respawn((f32, f32)),
    HitParticles((f32, f32)),
}

