use crate::gamemode::gamemode::Gamemodes;
use std::fmt::Display;
use std::io::{Read, Write};
use std::net::TcpStream;
use thiserror::Error;
#[repr(C)]
pub enum MessageType {
    PlayerPos,
    Shutdown,
    Undefined,
}
#[derive(Error, Debug)]
pub enum PlayerError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error), // Automatically converts io::Error to PlayerError::Io

    #[error("Player Disconnected")]
    PlayerDisconnected,

    #[error("Undefined packet recieved: {0}")]
    UndefinedPacket(u8),
}
pub struct PlayerMessage {
    pub message_type: MessageType,
    pub data: u8,
}
impl PlayerMessage {
    pub fn cast_buffer(buff: &[u8]) -> Self {
        let message_type = Self::decode_message(buff[0]);

        if let MessageType::Undefined = message_type {
            return Self {
                message_type,
                data: 0,
            };
        }
        Self {
            message_type,
            data: buff[1],
        }
    }
    pub fn decode_message(message_id: u8) -> MessageType {
        match message_id {
            0 => MessageType::PlayerPos,
            1 => MessageType::Shutdown,
            _ => MessageType::Undefined,
        }
    }
}

#[derive(Debug)]
pub struct NewPlayer {
    pub player_name: Option<String>,
    pub game_type: Gamemodes,
    pub tcp_stream: TcpStream,
}

impl NewPlayer {
    pub fn new(game_type: Gamemodes, tcp_stream: TcpStream) -> Self {
        Self {
            player_name: None,
            game_type,
            tcp_stream,
        }
    }
}

#[derive(Debug)]
pub struct Player {
    pub player_pos: u8,
    pub stream: TcpStream,
}

impl Player {
    pub fn from_new_player(new_player: NewPlayer) -> Self {
        Self {
            player_pos: 30,
            stream: new_player.tcp_stream,
        }
    }

    pub fn updated_position(&mut self, buff: &mut [u8]) -> Result<(), PlayerError> {
        let update_packet = self.stream.read(buff);

        match update_packet {
            Ok(n) => {
                if n == 0 {
                    return Err(PlayerError::PlayerDisconnected);
                }
                let player_message: PlayerMessage = PlayerMessage::cast_buffer(&buff[..n]);
                //
                match player_message.message_type {
                    MessageType::PlayerPos => {
                        self.player_pos = player_message.data;
                        Ok(())
                    }
                    MessageType::Shutdown => Err(PlayerError::PlayerDisconnected),
                    MessageType::Undefined => Err(PlayerError::UndefinedPacket(buff[0])),
                }
            }
            Err(e) => {
                return Err(PlayerError::Io(e));
            }
        }
    }

    pub fn send_hello(&mut self) {
        let _ = self.stream.write(b"Hello");
    }

    pub fn send_bytes(&mut self, message: &[u8]) {
        let _ = self.stream.write(message);
    }
}

impl Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.stream.peer_addr().unwrap())
    }
}
