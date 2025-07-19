use std::time::Duration;

use log::{debug, info, warn};
use rand::{Rng, seq::IndexedRandom};

use crate::models::player::{NewPlayer, Player, PlayerError};

#[derive(Debug)]
pub struct GameState {
    pub player_1: Player,
    pub player_2: Player,
    pub ball_pos_x: u8,
    pub ball_pos_y: u8,
    pub ball_dy: i8,
    pub ball_dx: i8,
    pub map_width: u8,
    pub map_height: u8,
    pub paddle_size: u8,
    pub player_1_score: u8,
    pub player_2_score: u8,
}

impl GameState {
    pub fn reset_ball_to_center(&mut self) {
        let mut rng = rand::rng();
        self.ball_pos_x = self.map_width / 2;
        self.ball_pos_y = self.map_height / 2;
        self.ball_dx = if rng.random_bool(0.5) { 1 } else { -1 };
        self.ball_dy = *[-1, 0, 1].choose(&mut rng).unwrap()
    }

    pub fn increment_score(&mut self, player_idx: u8) {
        if player_idx == 1 {
            self.player_1_score = self.player_1_score.wrapping_add(1);
        } else if player_idx == 2 {
            self.player_2_score = self.player_2_score.wrapping_add(1);
        }
    }
}
#[derive(Debug)]
pub enum Gamemodes {
    Standard,
}

pub trait Gamemode {
    fn setup_game(player_1: NewPlayer, player_2: NewPlayer) -> (Self, GameState)
    where
        Self: Sized;

    fn start_game(&mut self, gamestate: &mut GameState) -> i32;
    fn step_ball(&mut self, gamestate: &mut GameState);

    fn create_snapshot_packet(&self, gamestate: &mut GameState) -> Vec<u8> {
        let mut v_data: Vec<u8> = Vec::new();
        v_data.push(0);
        v_data.push(1);
        v_data.push(gamestate.player_1.player_pos);
        v_data.push(gamestate.player_2.player_pos);
        v_data.push(gamestate.ball_pos_x);
        v_data.push(gamestate.ball_pos_y);
        v_data.push(gamestate.map_width);
        v_data.push(gamestate.map_height);
        v_data.push(gamestate.paddle_size);
        v_data
    }

    fn create_update_packet(&self, gamestate: &mut GameState) -> Vec<u8> {
        let mut v_data: Vec<u8> = Vec::new();
        v_data.push(1);
        v_data.push(0);
        v_data.push(gamestate.player_1.player_pos);
        v_data.push(gamestate.player_2.player_pos);
        v_data.push(gamestate.ball_pos_x);
        v_data.push(gamestate.ball_pos_y);
        v_data
    }

    fn player_quit(&mut self, gamestate: &mut GameState) {
        todo!()
    }

    fn update_player_location(&mut self, gamestate: &mut GameState) -> Result<(), PlayerError> {
        let mut buff: [u8; 1024] = [0; 1024];
        let player_1_result = gamestate.player_1.updated_position(&mut buff);
        let player_2_result = gamestate.player_2.updated_position(&mut buff);

        match (player_1_result, player_2_result) {
            (Ok(_), Ok(_)) => Ok(()),
            (Ok(_), Err(e)) => Err(e),
            (Err(e), Ok(_)) => Err(e),
            (Err(e), Err(e2)) => Err(e),
        }
    }

    fn reset_ball_pos(&mut self, gamestate: &mut GameState) {
        gamestate.ball_pos_x = gamestate.map_width / 2;
        gamestate.ball_pos_y = gamestate.map_height / 2;
    }

    fn new_ball_pos(&mut self, gamestate: &mut GameState) -> (u8, u8) {
        (
            (gamestate.ball_pos_x as i16 + gamestate.ball_dx as i16) as u8,
            (gamestate.ball_pos_y as i16 + gamestate.ball_dy as i16) as u8,
        )
    }

    fn calculate_next_frame(&mut self, gamestate: &mut GameState) {
        match gamestate.ball_pos_x {
            x if x == 1 => {
                gamestate.player_2_score = gamestate.player_2_score.wrapping_add(1);
                gamestate.reset_ball_to_center();
            }

            x if x == gamestate.map_width - 3 => {
                gamestate.player_1_score = gamestate.player_1_score.wrapping_add(1);
                gamestate.reset_ball_to_center();
            }

            _ => {}
        }
    }

    fn print_game_state(&self, gamestate: &mut GameState) {
        debug!(
            "{}:{} Ball x: {} Ball y: {} Ball DX: {} Ball DY: {} P1 Pos: {} P2 Pos: {} Map Width: {} Map Height {}",
            gamestate.player_1_score,
            gamestate.player_2_score,
            gamestate.ball_pos_x,
            gamestate.ball_pos_y,
            gamestate.ball_dx,
            gamestate.ball_dy,
            gamestate.player_1.player_pos,
            gamestate.player_2.player_pos,
            gamestate.map_width,
            gamestate.map_height
        )
    }
}
