use std::{
    io::Write,
    net::TcpStream,
    sync::mpsc::Receiver,
    thread::{self},
    time::Duration,
    vec,
};

use super::game_drawer::{GameRender, Tile};
use crossterm::{
    event::{Event, poll, read},
    terminal::disable_raw_mode,
};
use log::{debug, warn};
#[derive(Debug)]
pub struct GameUpdate {
    pub packet_id: u8,
    pub player_id: u8,
    pub player_1_pos: u8,
    pub player_2_pos: u8,
    pub ball_x: u8,
    pub ball_y: u8,
}

impl GameUpdate {
    pub fn cast_packet(buf: &[u8]) -> Self {
        let packet_id = buf[0];
        let player_id = buf[1];
        let player_1 = buf[2];
        let player_2 = buf[3];
        let ball_pos_x = buf[4];
        let ball_pos_y = buf[5];

        Self {
            packet_id,
            player_id,
            player_1_pos: player_1,
            player_2_pos: player_2,
            ball_x: ball_pos_x,
            ball_y: ball_pos_y,
        }
    }
}

pub struct PlayerUpdate {
    pub packet_type: MessageType,
    pub data: u8,
}

impl PlayerUpdate {
    pub fn position_update(position_i16: i32) -> Self {
        let data = position_i16.to_le_bytes()[0];
        Self {
            packet_type: MessageType::PlayerPos,
            data,
        }
    }
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut v_data: Vec<u8> = Vec::new();
        v_data.push(self.packet_type.clone() as u8);
        v_data.push(self.data);
        v_data
    }
}
#[derive(Clone)]
#[repr(u8)]
pub enum MessageType {
    PlayerPos = 0,
    Shutdown = 1,
}

#[derive(Debug)]
pub struct Game {
    reader_pipe: Receiver<GameUpdate>,
    writer_stream: TcpStream,
    packet_id: u8,
    player_id: u8,
    player_1_y: i32,
    player_2_y: i32,
    ball_pos_x: u8,
    ball_pos_y: u8,
    paddle_size: i32,
    map_width: i32,
    map_height: i32,
    map: Vec<Tile>,
}
impl Game {
    pub fn new(reader_pipe: Receiver<GameUpdate>, writer_stream: TcpStream) -> Self {
        Self {
            reader_pipe: reader_pipe,
            writer_stream: writer_stream,
            packet_id: 0,
            player_id: 0,
            player_1_y: 0,
            player_2_y: 0,
            ball_pos_x: 0,
            ball_pos_y: 0,
            paddle_size: 0,
            map_width: 0,
            map_height: 0,
            map: vec![],
        }
    }
    pub fn initialize_game(&mut self, buf: &[u8]) {
        debug!("Init packet: {:?}", buf);
        let packet_id = buf[0];
        let player_id = buf[1];
        let player_1: i32 = buf[2].into();
        let player_2: i32 = buf[3].into();

        let ball_pos_x = buf[4];
        let ball_pos_y = buf[5];
        let map_width: i32 = buf[6].into();
        let map_height: i32 = buf[7].into();
        let paddle_size = buf[8].into();
        let mut map = vec![Tile::Empty; (map_height * map_width) as usize];

        for y in 0..map_height {
            for x in 0..map_width {
                if x == 0 || x == map_width - 1 {
                    map[(y * map_width + x) as usize] = Tile::VerticalWall;
                } else if y == 0 || y == map_height - 1 {
                    map[(y * map_width + x) as usize] = Tile::HorizontalWall;
                }

                if y == 5 {
                    // map[(y * map_width + x) as usize] = Tile::Debug
                }
            }
        }
        debug!("Building paddles");
        debug!(
            "{} {} {} {:?} {:?}",
            player_1,
            player_2,
            paddle_size,
            (player_1 - paddle_size)..(player_1 + paddle_size),
            (player_2 - paddle_size)..(player_2 + paddle_size)
        );
        debug!(
            "Map Bounds: {} {} Max Array: {}",
            map_width,
            map_height,
            map_width * map_height
        );
        for paddle_pos in (player_2 - paddle_size)..(player_2 + paddle_size) {
            debug!("Player 2 Paddle {}", paddle_pos);
            map[(paddle_pos * map_width + (map_width - 3)) as usize] = Tile::Player;
            // map[(paddle_pos * map_width + 3) as usize] = Tile::Player;
        }
        for paddle_pos in (player_1 - paddle_size)..(player_1 + paddle_size + 1) {
            debug!(
                "Player 1 Paddle {} {:?}",
                paddle_pos,
                (paddle_pos, map_height - 2)
            );
            map[(paddle_pos * map_width + 2) as usize] = Tile::Player;
        }
        // debug!("")
        // let index =
        //     ((self.map_height - 3) * self.map_width + self.player_1_x - paddle_size) as usize;
        // map[index] = Tile::Player;
        map[(0 * map_width + 0) as usize] = Tile::Corner;
        map[((map_height - 1) * map_width + 0) as usize] = Tile::Corner;
        map[(0 * map_width + (map_width - 1)) as usize] = Tile::Corner;
        map[((map_height - 1) * map_width + (map_width - 1)) as usize] = Tile::Corner;

        // debug!("Le number {}", 0 * map_width + 3);
        // debug!("Le number{}", 1 * map_width + 3);
        // map[(1 * map_width + 3) as usize] = Tile::Debug;

        self.packet_id = packet_id;
        self.player_id = player_id;
        self.player_1_y = player_1.into();
        self.player_2_y = player_2.into();
        self.ball_pos_x = ball_pos_x;
        self.ball_pos_y = ball_pos_y;
        self.map_width = map_width.into();
        self.map_height = map_height.into();
        self.paddle_size = paddle_size;
        self.map = map;
        // debug!("Setup complete {:?}", self)
    }
    pub fn key_stroke_move(
        &mut self,
        event: crossterm::event::KeyEvent,
    ) -> std::result::Result<(), ()> {
        match event.code {
            crossterm::event::KeyCode::Backspace => {
                disable_raw_mode().unwrap();
                Err(())
            }
            // crossterm::event::KeyCode::Left => {
            //     // debug!("Left arrow pressed, going right");
            //     if self.player_1_y - PADDLE_SIZE - 1 > 0 {
            //         self.player_1_y -= 1;
            //         self.map[((self.map_height - 3) * self.map_width
            //             + (self.player_1_y + self.paddle_size))
            //             as usize] = Tile::Empty;

            //         self.map[((self.map_height - 3) * self.map_width
            //             + (self.player_1_y - self.paddle_size))
            //             as usize] = Tile::Player;
            //     }
            //     debug!("Left key input");
            //     Ok(())
            // }
            // crossterm::event::KeyCode::Right => {
            //     debug!("Right arrow pressed, going right");
            //     if self.player_1_y + PADDLE_SIZE - 1 < self.map_width - 1 {
            //         self.player_1_y += 1;
            //         self.map[((self.map_height - 3) * self.map_width
            //             + (self.player_1_y + self.paddle_size))
            //             as usize] = Tile::Player;

            //         self.map[((self.map_height - 3) * self.map_width
            //             + (self.player_1_y - self.paddle_size))
            //             as usize] = Tile::Empty;
            //     }
            //     debug!("Right key input");
            //     Ok(())
            // }
            crossterm::event::KeyCode::Up => {
                debug!("Up arrow pressed, going up");
                if self.player_1_y - self.paddle_size - 1 > 0 {
                    self.player_1_y -= 1;
                    self.map
                        [((self.player_1_y - self.paddle_size) * self.map_width + 2) as usize] =
                        Tile::Player;

                    self.map[((self.player_1_y + self.paddle_size + 1) * self.map_width + 2)
                        as usize] = Tile::Empty;
                    // self.map[((self.map_height - 3) * self.map_width
                    //     + (self.player_1_y - self.paddle_size))
                    //     as usize] = Tile::Empty;
                }

                Ok(())
            }
            crossterm::event::KeyCode::Down => {
                debug!("Down arrow pressed, going down");
                if self.player_1_y + self.paddle_size + 1 < self.map_height - 1 {
                    self.player_1_y += 1;
                    self.map[((self.player_1_y - self.paddle_size - 1) * self.map_width + 2)
                        as usize] = Tile::Empty;

                    self.map
                        [((self.player_1_y + self.paddle_size) * self.map_width + 2) as usize] =
                        Tile::Player;
                    // self.map[((self.map_height - 3) * self.map_width
                    //     + (self.player_1_y - self.paddle_size))
                    //     as usize] = Tile::Empty;
                }

                Ok(())
            }
            _ => Ok(()),
        }
    }
    pub fn update_game_state(&mut self, game_update: GameUpdate) {
        let opponent_prev_pos: i32;
        let opponent_new_pos: i32;

        if self.player_id == 1 {
            opponent_prev_pos = self.player_2_y;
            opponent_new_pos = game_update.player_2_pos.into();
            self.player_2_y = game_update.player_2_pos.into();
        } else {
            opponent_prev_pos = self.player_2_y;
            opponent_new_pos = game_update.player_1_pos.into();
            self.player_2_y = game_update.player_1_pos.into();
        }

        self.map[(self.ball_pos_y as i32 * self.map_width + self.ball_pos_x as i32) as usize] =
            Tile::Empty;
        self.ball_pos_x = game_update.ball_x;
        self.ball_pos_y = game_update.ball_y;
        self.map[(self.ball_pos_y as i32 * self.map_width + self.ball_pos_x as i32) as usize] =
            Tile::Ball;

        let new_paddle_range =
            (opponent_new_pos - self.paddle_size)..=(opponent_new_pos + self.paddle_size);

        let old_paddle_range =
            (opponent_prev_pos - self.paddle_size)..=(opponent_prev_pos + self.paddle_size);

        // map[(paddle_pos * map_width + map_width - 3) as usize] = Tile::Player;

        for paddle_tile in old_paddle_range {
            if !new_paddle_range.contains(&paddle_tile) {
                self.map[(paddle_tile * self.map_width + self.map_width - 3) as usize] =
                    Tile::Empty;
                // let index = (paddle_tile * self.map_width + self.map_width - 3) as usize;
                // let tile = self.map.get_mut(index);

                // if let Some(tile) = tile {
                //     if *tile == Tile::Player {
                //         *tile = Tile::Empty;
                //     }
                // }
            }
        }

        for tile_y in new_paddle_range {
            let index = (tile_y * self.map_width + self.map_width - 3) as usize;
            let tile = self.map.get_mut(index);
            if let Some(tile) = tile {
                if *tile == Tile::Empty {
                    *tile = Tile::Player;
                }
            } else {
                warn!("Failed to get tile as mutable")
            }
        }
    }

    pub fn draw_matrix(&mut self) {
        println!("{}", self.map.len());
        for y in 0..self.map_height {
            let mut matrix_string: String = String::new();
            for x in 0..self.map_width {
                // string.
                let value = &self.map[(y * self.map_width + x) as usize];
                matrix_string += &format!("{:?}", value.clone() as u8);
            }
            println!("{}", matrix_string)
        }
        println!("")
    }
    pub fn start_game(&mut self) {
        let mut game_render = GameRender::setup_renderer(self.map_height, self.map_width);
        let delay = Duration::from_millis(10);
        'main_loop: loop {
            let game_update = self.reader_pipe.recv().unwrap();
            debug!("Game Update recved{:?}", game_update);
            self.update_game_state(game_update);
            if let Ok(key_pressed) = poll(Duration::from_millis(1)) {
                if key_pressed {
                    match read().unwrap() {
                        Event::Key(event) => {
                            let player_move = self.key_stroke_move(event);
                            match player_move {
                                Ok(_) => {
                                    debug!("Received input :: {:?}", event);
                                }
                                Err(e) => {
                                    warn!("Error occurred getting input: {:?}. Exiting loop.", e);
                                    break 'main_loop;
                                }
                            }
                        }
                        _ => {}
                    };
                } else {
                    // No key pressed
                }
            };

            // disable_raw_mode();
            // self.draw_matrix();
            // enable_raw_mode();
            game_render.render_game(&self.map, self.player_id);

            let bytes = self.player_1_y.to_le_bytes();
            debug!("Bytes {:?}", bytes);
            let pos_update_packet: PlayerUpdate = PlayerUpdate::position_update(self.player_1_y);
            let writer_result = self.writer_stream.write(&pos_update_packet.as_bytes());
            match writer_result {
                Ok(n) => {
                    if n == 0 {
                        break;
                    }
                }
                Err(_) => todo!(),
            }
            thread::sleep(delay);
        }
        debug!("Stopping game");
        drop(game_render);
    }
}

impl Drop for Game {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
    }
}
