use log::{debug, info, warn};

use crate::{
    gamemode::gamemode::GameState,
    models::player::{NewPlayer, Player, PlayerError},
};

use super::gamemode::Gamemode;
use rand::{Rng, seq::IndexedRandom};
use std::{thread, time::Duration};

#[derive(Debug)]
pub struct StandardGame {
    stepping: bool,
}

impl StandardGame {}

impl Gamemode for StandardGame {
    fn setup_game(player_1: NewPlayer, player_2: NewPlayer) -> (Self, GameState) {
        let mut player_1 = Player::from_new_player(player_1);
        let mut player_2 = Player::from_new_player(player_2);

        const MAP_WIDTH: u8 = 80;
        const MAP_HEIGHT: u8 = 30;
        const PADDLE_SIZE: u8 = 4;
        const PLAYER_TIMEOUT: Duration = Duration::from_millis(25);

        let ball_start_x = MAP_WIDTH / 2;
        let ball_start_y = MAP_HEIGHT / 2;

        player_1.player_pos = MAP_HEIGHT / 2;
        player_2.player_pos = MAP_HEIGHT / 2;

        player_1
            .stream
            .set_read_timeout(Some(PLAYER_TIMEOUT))
            .unwrap();
        player_2
            .stream
            .set_read_timeout(Some(PLAYER_TIMEOUT))
            .unwrap();

        let gamemode_logic = StandardGame { stepping: false };

        let initial_game_state = GameState {
            player_1: player_1,
            player_2: player_2,
            ball_pos_x: ball_start_x,
            ball_pos_y: ball_start_y,
            ball_dy: 0,
            ball_dx: 1,
            map_width: MAP_WIDTH,
            map_height: MAP_HEIGHT,
            paddle_size: PADDLE_SIZE,
            player_1_score: 0,
            player_2_score: 0,
        };

        (gamemode_logic, initial_game_state)
    }

    fn start_game(&mut self, gamestate: &mut GameState) -> i32 {
        info!(
            "Starting game {} vs {}",
            gamestate.player_1, gamestate.player_2
        );

        debug!("Sending game snapshot");
        let mut snapshot_packet = self.create_snapshot_packet(gamestate);
        gamestate.player_1.send_bytes(&snapshot_packet);
        snapshot_packet[1] = 2;
        gamestate.player_2.send_bytes(&snapshot_packet);
        let mut update_packet = self.create_update_packet(gamestate);
        gamestate.player_1.send_bytes(&update_packet);
        update_packet[1] = 2;
        gamestate.player_2.send_bytes(&update_packet);
        loop {
            debug!("Sending snapshot");
            update_packet = self.create_update_packet(gamestate);
            update_packet[1] = 1;
            gamestate.player_1.send_bytes(&update_packet);
            update_packet[1] = 2;
            update_packet[4] = gamestate.map_width - update_packet[4] - 1;
            gamestate.player_2.send_bytes(&update_packet);

            let update_result = self.update_player_location(gamestate);
            match update_result {
                Ok(_) => {
                    self.calculate_next_frame(gamestate);
                    if self.stepping {
                        self.step_ball(gamestate);
                        self.stepping = false;
                    } else {
                        self.stepping = true;
                    }
                    self.print_game_state(gamestate);
                }
                Err(e) => match e {
                    PlayerError::Io(error) => todo!("Player IO Error {:?}", error),
                    PlayerError::PlayerDisconnected => todo!("Player Disconnected"),
                    PlayerError::UndefinedPacket(n) => todo!("Undefined Packet Number: {}", n),
                },
            }
            thread::sleep(Duration::from_millis(35));
        }
    }

    fn step_ball(&mut self, gamestate: &mut GameState) -> () {
        let mut rng = rand::rng();
        //
        let (mut new_x, mut new_y) = self.new_ball_pos(gamestate);

        let player_1_paddle = (gamestate.player_1.player_pos - gamestate.paddle_size)
            ..(gamestate.player_1.player_pos + gamestate.paddle_size + 1);

        let player_2_paddle = (gamestate.player_2.player_pos - gamestate.paddle_size)
            ..(gamestate.player_2.player_pos + gamestate.paddle_size + 1);

        // Check wall + paddle p1 colision
        // Check wall + paddle p2 colision
        // Check wall collision
        // Check paddle collision
        let mut collison_detected: bool = false;
        match (new_x, new_y) {
            // p1 paddle +wall collision
            (x, y)
                if (y == 0 || y == gamestate.map_height - 1)
                    && (x == 2 || x == gamestate.map_width - 3) =>
            {
                let random_change = rng.random_range(-1..0);
                gamestate.ball_dx = gamestate.ball_dx * -1;
                gamestate.ball_dy = random_change * gamestate.ball_dy;
                collison_detected = true;
            }

            (x, y)
                if (x == 2 && (player_1_paddle.contains(&y)))
                    || (x == gamestate.map_width - 3 && (player_2_paddle.contains(&y))) =>
            {
                info!("Ball his hit player 1 paddle",);
                gamestate.ball_dy = match gamestate.ball_dx {
                    -1 => *[0, 1].choose(&mut rng).unwrap(),
                    0 => *[-1, 0, 1].choose(&mut rng).unwrap(),
                    1 => *[-1, 0].choose(&mut rng).unwrap(),
                    _ => 0,
                };
                gamestate.ball_dx = gamestate.ball_dx * -1;
                collison_detected = true;
            }

            (_, y) if y == gamestate.map_height - 1 || y == 0 => {
                gamestate.ball_dy = gamestate.ball_dy * -1;
                collison_detected = true;
            }

            _ => {
                // No collision
            }
        }
        if collison_detected {
            (new_x, new_y) = self.new_ball_pos(gamestate);
        }

        gamestate.ball_pos_x = new_x;
        gamestate.ball_pos_y = new_y;
    }
}
