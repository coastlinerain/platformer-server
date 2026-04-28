use crossbeam_channel::Sender;
use laminar::{Config, Packet, Socket, SocketEvent};
use std::{collections::HashMap, net::SocketAddr, time::Instant};
mod game_packet;
use game_packet::GamePacket;

fn main() {
    let mut socket = Socket::bind("127.0.0.1:12345").unwrap();
    let packet_sender = socket.get_packet_sender();
    let event_receiver = socket.get_event_receiver();

    let _thread = std::thread::spawn(move || socket.start_polling());

    let mut next_id = 1;
    let mut players = HashMap::new();

    println!("Servidor de Rustvania corriendo en el puerto 12345...");

    loop {
        if let Ok(event) = event_receiver.recv() {
            match event {
                SocketEvent::Packet(packet) => {
                    let addr = packet.addr();

                    // 1. DESERIALIZAR: Convertir bytes a nuestro Enum
                    if let Ok(decoded) = postcard::from_bytes::<GamePacket>(packet.payload()) {
                        match decoded {
                            GamePacket::JoinRequest => {
                                let id = next_id;
                                players.insert(addr, id);
                                next_id += 1;

                                let response = GamePacket::JoinResponse { assigned_id: id };
                                let bytes = postcard::to_allocvec(&response).unwrap();

                                packet_sender
                                    .send(Packet::reliable_ordered(addr, bytes, None))
                                    .unwrap();
                                println!("Jugador {} unido desde {}", id, addr);
                            }
                            GamePacket::PlayerPos {
                                id,
                                x,
                                y,
                                dir,
                                level_x,
                                level_y,
                            } => {
                                for (&player_addr, &player_id) in players.iter() {
                                    if player_addr != addr {
                                        let bytes = postcard::to_allocvec(&GamePacket::PlayerPos {
                                            id,
                                            x,
                                            y,
                                            dir,
                                            level_x,
                                            level_y,
                                        })
                                        .unwrap();
                                        packet_sender
                                            .send(Packet::unreliable(player_addr, bytes))
                                            .unwrap();
                                    }
                                }
                            }
                            GamePacket::Action { id, kind, dir } => {
                                if kind == "shoot" {
                                    println!("Jugador {} disparó en dirección {}", id, dir);

                                    for (&player_addr, &player_id) in players.iter() {
                                        if player_id != id {
                                            let bytes =
                                                postcard::to_allocvec(&GamePacket::Action {
                                                    id,
                                                    kind: kind.clone(),
                                                    dir,
                                                })
                                                .unwrap();
                                            packet_sender
                                                .send(Packet::reliable_ordered(
                                                    player_addr,
                                                    bytes,
                                                    None,
                                                ))
                                                .unwrap();
                                        }
                                    }
                                }
                            }
                            GamePacket::Leave { id } => {
                                println!("Jugador {} solicitó salir voluntariamente", id);
                                players.remove(&addr); // Limpiamos nuestro registro

                                broadcast_leave(&players, &packet_sender, id);
                            }
                            GamePacket::Hit { id } => {
                                println!("Jugador {} ha muerto", id);

                                // Retransmitimos a todos para que actualicen sus listas
                                for (&player_addr, &player_id) in players.iter() {
                                    if player_id != id {
                                        let bytes =
                                            postcard::to_allocvec(&GamePacket::Hit { id: (id) })
                                                .unwrap();
                                        packet_sender
                                            .send(Packet::reliable_ordered(
                                                player_addr,
                                                bytes,
                                                None,
                                            ))
                                            .unwrap();
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                SocketEvent::Timeout(addr) => {
                    if let Some(id) = players.remove(&addr) {
                        println!("Jugador {} desconectado", id);
                        broadcast_leave(&players, &packet_sender, id);
                    }
                }
                _ => {}
            }
        }
    }
}

fn broadcast_leave(players: &HashMap<SocketAddr, u8>, sender: &Sender<Packet>, dropped_id: u8) {
    let leave_msg = GamePacket::Leave { id: dropped_id };
    let bytes = postcard::to_allocvec(&leave_msg).expect("Error serializando Leave");

    for &target_addr in players.keys() {
        // Creamos un paquete para cada destino
        let packet = Packet::reliable_ordered(target_addr, bytes.clone(), None);
        if let Err(e) = sender.send(packet) {
            eprintln!(
                "Error enviando broadcast de salida a {}: {}",
                target_addr, e
            );
        }
    }
}
