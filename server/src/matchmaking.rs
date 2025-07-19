use std::{collections::VecDeque, io::Read, sync::mpsc::Receiver, thread};

use log::debug;

use crate::{
    gamemode::{gamemode::Gamemode, standard::StandardGame},
    models::player::NewPlayer,
};

pub struct MatchMaker {
    client_rx: Receiver<NewPlayer>,
    player_queue: VecDeque<NewPlayer>,
}

impl MatchMaker {
    pub fn new(client_rx: Receiver<NewPlayer>) -> Self {
        Self {
            client_rx,
            player_queue: VecDeque::new(),
        }
    }

    pub fn recieve_new_player(&mut self) {
        while let Ok(mut player) = self.client_rx.recv() {
            Self::setup_player(&mut player);
            debug!("Recieved player: {:?}", player);
            let _ = self.player_queue.push_back(player);

            if self.player_queue.len() > 1 {
                println!("Starting game");
                let player_1 = self.player_queue.pop_front().unwrap();
                let player_2 = self.player_queue.pop_front().unwrap();

                let (mut gamelogic, mut gamestate) = StandardGame::setup_game(player_1, player_2);
                let _ = thread::Builder::new()
                    .name("Game".to_string())
                    .spawn(move || gamelogic.start_game(&mut gamestate));
            }
        }
    }

    pub fn setup_player(player: &mut NewPlayer) {
        let mut buff: [u8; 1024] = [0; 1024];
        let n = player.tcp_stream.read(&mut buff).unwrap();
        player.player_name = Some(buff[..n].iter().map(|c| *c as char).collect::<String>())
    }
}
