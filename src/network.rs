use std::{net::{TcpStream, TcpListener, SocketAddr}, io::{ErrorKind, Write, BufReader}, collections::HashMap};
use macroquad::prelude::*;
use serde::{Serialize, Deserialize};

use crate::Player;

const PORT: u16 = 5356;

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

struct ServerConnection {
    reader: BufReader<TcpStream>,
    state: State,
    addr: SocketAddr,
    user_name: String,
}

pub struct Server {
    listener: TcpListener,
    clients: Vec<ServerConnection>,
    naslednji_id: u32,
    user_name: String,
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

    fn handle_msg(&mut self, msg: Message, conn_i: usize) {
        match msg {
            Message::PlayerState(state) => {
                let id = self.clients[conn_i].state.id;
                self.clients[conn_i].state = state;
                self.clients[conn_i].state.id = id;
            },
            Message::UserName((_id, name)) => {
                self.clients[conn_i].user_name = name.clone();
                let msg = Message::UserName((self.clients[conn_i].state.id, name));
                let send_buf = bincode::serialize(&msg).unwrap();
                self.send_to_all(&send_buf);
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

    pub fn narisi_cliente(&self, tekstura: &Texture2D) {
        for conn in &self.clients {
            //draw_rectangle_lines(conn.state.position.0, conn.state.position.1, 16.0, 28.0, 1.0, macroquad::color::RED);
            let state = &conn.state;
            Player::narisi_iz(tekstura, state.position.into(), state.anim_frame.into(), state.rotation, state.attack_time, &conn.user_name);
        }
    }

    pub fn poslji_vse_state(&mut self, player: &Player) {
        let mut states: Vec<State> = self.clients.iter()
            .map(|c| c.state.clone())
            .collect();
        states.push(State {
            id: 0, // gazda/host id
            position: (player.position.x, player.position.y),
            rotation: player.get_rotation(),
            anim_frame: player.get_anim().izr_frame_xy().into(),
            attack_time: player.attack_time,
        });
        let send_buf = bincode::serialize(&Message::AllPlayersState(states)).unwrap();
        self.send_to_all(&send_buf);
    }
}

pub struct Client {
    pub id: u32,
    reader: BufReader<TcpStream>,
    net_states: Vec<State>,
    net_user_names: HashMap<u32, String>,
}

impl Client {
    pub fn new(addr: &str, name: String) -> Client {
        let mut stream = TcpStream::connect((addr, PORT)).unwrap();
        prepare_socket(&mut stream);
        let msg = Message::UserName((u32::MAX, name));
        stream.write(&bincode::serialize(&msg).unwrap()).unwrap();
        Client {
            id: u32::MAX,
            reader: BufReader::new(stream),
            net_states: Vec::new(),
            net_user_names: HashMap::new(),
        }
    }

    pub fn send(&mut self, bytes: &[u8]) {
        self.reader.get_mut().write(bytes).unwrap();
    }

    pub fn handle_msg(&mut self, msg: Message) {
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
            _ => {},
        }
    }

    pub fn recv(&mut self) {
        loop {
            match bincode::deserialize_from::<&mut BufReader<TcpStream>, Message>(&mut self.reader) {
                Ok(msg) => {
                    //println!("recv: {:?}", msg);
                    self.handle_msg(msg);
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
            Player::narisi_iz(tekstura, state.position.into(), state.anim_frame.into(), state.rotation, state.attack_time, name);
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
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    DodeljenId(u32),
    UserName((u32, String)),
    PlayerState(State),
    AllPlayersState(Vec<State>),
}

