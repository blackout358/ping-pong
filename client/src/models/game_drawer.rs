use crossterm::{
    ExecutableCommand, QueueableCommand, cursor,
    style::{self, Stylize},
    terminal::{self, disable_raw_mode, enable_raw_mode},
};
use log::{debug, info};
use std::io::{Stdout, Write, stdout};

#[repr(u8)]
#[derive(Clone, Debug, PartialEq)]
pub enum Tile {
    Debug = 6,
    Corner = 5,
    HorizontalWall = 4,
    VerticalWall = 3,
    Ball = 1,
    Player = 2,
    Empty = 0,
}
pub struct GameRender {
    cursor_y: u16,
    cursor_x: i32,
    game_width: i32,
    game_height: i32,
    stdout: Stdout,
}
// 2d array = [u8; wd * height];
// val = [y * map_width + x]
impl GameRender {
    pub fn setup_renderer(map_height: i32, map_width: i32) -> Self {
        enable_raw_mode().unwrap();

        Self {
            cursor_y: 0,
            cursor_x: 0,

            game_width: map_width,
            game_height: map_height,
            stdout: stdout(),
        }
    }

    pub fn render_game(&mut self, game_map: &Vec<Tile>, player_id: u8) {
        info!("Render Game");
        self.stdout
            .execute(terminal::Clear(terminal::ClearType::All))
            .unwrap();
        self.cursor_reset();
        for y in 0..self.game_height {
            for x in 0..self.game_width {
                let cell = &game_map[(y * self.game_width + x) as usize];
                let char_to_render = match cell {
                    Tile::Player => "█".white(), // Wall
                    Tile::Ball => "█".blue(),    // Food
                    Tile::Corner => "+".white(),
                    Tile::VerticalWall => "|".white(),
                    Tile::HorizontalWall => "-".white(),
                    Tile::Debug => "█".dark_magenta(),
                    _ => " ".dark_grey(),
                };
                self.stdout
                    .queue(style::PrintStyledContent(char_to_render))
                    .unwrap();
            }
            self.cursor_newline();
        }
        self.cursor_newline();
        self.cursor_newline();

        let _ = self.stdout.queue(style::PrintStyledContent(
            format!("Player {}", player_id).as_str().cyan(),
        ));
        self.cursor_newline();
        self.cursor_newline();
        self.stdout.flush().unwrap();
    }

    pub fn cursor_reset(&mut self) {
        self.stdout.queue(cursor::MoveTo(0, 0)).unwrap();
        self.cursor_y = 0;
        self.cursor_x = 0;
    }

    pub fn cursor_newline(&mut self) {
        self.cursor_y += 1;
        self.stdout.queue(cursor::MoveTo(0, self.cursor_y)).unwrap();
    }
}
impl Drop for GameRender {
    fn drop(&mut self) {
        debug!("Gamerender being dropped");
        disable_raw_mode().unwrap();
        // Perform cleanup here
    }
}
