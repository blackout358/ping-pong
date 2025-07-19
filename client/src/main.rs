use std::{
    env,
    io::{Read, Write},
    net::TcpStream,
    sync::mpsc::{self, Sender},
    thread::{self, JoinHandle},
};
// use chrono;
pub mod models;

use crossterm::terminal::disable_raw_mode;
use flexi_logger::{FileSpec, Logger};
use log::{Level, debug, info, warn};
use models::game::{Game, GameUpdate};
const SERVER_ADDRESS: &str = "127.0.0.1:9090";

fn init_logger() {
    let file_spec = FileSpec::default()
        .directory("./")
        .basename("Client Runtime")
        .suppress_timestamp()
        .suffix("log");

    Logger::try_with_str("debug")
        .unwrap()
        .log_to_file(file_spec)
        .format(|writer, now, record| {
            let timestamp = now.now().format("%Y-%m-%d %H:%M:%S");
            // Colorize level
            // let level = match record.level() {
            //     Level::Error => "ERROR".red(),
            //     Level::Warn => "WARN ".yellow(),
            //     Level::Info => "INFO ".green(),
            //     Level::Debug => "DEBUG".blue(),
            //     Level::Trace => "TRACE".blue(),
            // };
            let level = match record.level() {
                Level::Error => "ERROR",
                Level::Warn => "WARN ",
                Level::Info => "INFO ",
                Level::Debug => "DEBUG",
                Level::Trace => "TRACE",
            };
            // Get thread name or fallback
            let binding = std::thread::current();
            let thread = binding.name().unwrap_or("unnamed");

            let line = record.line().map_or("".to_string(), |l| format!("L{}", l));
            let module = record.module_path().unwrap_or("?");
            // let asd = record.

            write!(
                writer,
                "{} :: {} :: {}::{}() :: {} :: {}",
                timestamp,
                level,
                thread,
                module,
                line,
                record.args()
            )
        })
        .start()
        .unwrap();
}
fn main() {
    // println!("Hello, world!");
    let args: Vec<String> = env::args().collect();

    let username: String;

    if args.len() > 1 {
        username = args[1].clone();
    } else {
        username = "Default Name".to_string();
    }

    // env_logger::init();
    init_logger();
    let tcp_connection = TcpStream::connect(SERVER_ADDRESS).unwrap();
    info!("Connected to server {}", SERVER_ADDRESS);
    let mut buf: [u8; 1024] = [0; 1024];
    let mut reader_stream = tcp_connection.try_clone().unwrap();
    let mut writer_stream = tcp_connection;
    let _ = writer_stream.write(username.as_bytes());

    let mut pipe_sender: Option<Sender<GameUpdate>> = None;
    let mut _game_thread_handler: Option<JoinHandle<()>> = None;
    loop {
        debug!("Reading from stream");
        match reader_stream.read(&mut buf) {
            Ok(0) => {
                // 0 bytes read = EOF, connection closed
                debug!("Connection closed.");
                break;
            }
            Ok(n) => {
                debug!("Received: {:?}", &buf[..n]);
                if buf[0] == 0 {
                    // println!("{} ", n);
                    let (tx, rx) = mpsc::channel::<GameUpdate>();
                    pipe_sender = Some(tx);

                    let mut game = Game::new(rx, writer_stream.try_clone().unwrap());
                    game.initialize_game(&buf[..n]);
                    // game.draw_matrix();

                    let game_thread = thread::Builder::new()
                        .name("Game Thread".to_string())
                        .spawn(move || {
                            game.start_game();
                        })
                        .unwrap();

                    debug!("Game thread started :: {:?}", game_thread);

                    _game_thread_handler = Some(game_thread);

                    // }
                } else if buf[0] == 1 {
                    let game_update = GameUpdate::cast_packet(&buf[..n]);
                    if let Some(pipe) = &pipe_sender {
                        match pipe.send(game_update) {
                            Ok(_) => debug!("Pipe sent successfully"),
                            Err(r) => {
                                warn!("Error sending game update {:?}", r);
                            }
                        }
                    } else {
                        warn!("Trying to send on non existing pipe");
                    }
                }
            }
            Err(e) => {
                warn!("Error occurred {:?}", e);
                // disable_raw_mode().unwrap();
                break;
            }
        }
    }
    disable_raw_mode().unwrap();
}
