pub mod gamemode;
pub mod logger_setup;
pub mod matchmaking;
pub mod models;

use std::{net::TcpListener, thread};

use gamemode::gamemode::Gamemodes;
use log::info;
use matchmaking::MatchMaker;
use models::player::NewPlayer;
use std::sync::mpsc::channel;

const SERVER_ADDRESS: &str = "127.0.0.1:9090";

fn main() {
    let tcp_listener = TcpListener::bind(SERVER_ADDRESS).unwrap();
    // env_logger::init();
    logger_setup::init_logger();
    info!("Listening to {}", SERVER_ADDRESS);
    let (tx, rx) = channel::<NewPlayer>();

    let mut match_making = MatchMaker::new(rx);

    let _match_making_listener = thread::Builder::new()
        .name("Matchmaking".to_string())
        .spawn(move || match_making.recieve_new_player());
    for stream in tcp_listener.incoming() {
        match stream {
            Ok(stream) => {
                // thread::spawn(move || handle_stream(stream));
                let new_player = NewPlayer::new(Gamemodes::Standard, stream);
                let _ = tx.send(new_player);
            }
            Err(e) => println!("Error Occured: {:?}", e),
        }
    }
}
