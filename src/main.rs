mod engine;
mod eval;
mod stats;
mod tables;
mod tests;
mod time;
mod tt;
mod utils;

use std::io;
use std::io::BufRead;
use std::str::FromStr;

use chess::Board;
use engine::Engine;
use time::TimeManager;
use utils::History;
use vampirc_uci::parse_one;
use vampirc_uci::UciMessage;

fn main() {
    let mut tt_size_mb = 256;
    let mut info = false;

    let mut board = Board::default();
    let mut eng = Engine::new(tt_size_mb);
    let mut hist = History::new();

    for line in io::stdin().lock().lines() {
        let msg: UciMessage = parse_one(&line.expect("Parse UCI message"));
        match msg {
            UciMessage::Uci => {
                println!("id name PiCa v{}", env!("CARGO_PKG_VERSION"));
                println!("id author Filip Rusz <filip@rusz.space>");

                // List options
                println!("option name Hash type spin default 256 min 1 max 8192");
                println!("option name Info type check default false");

                println!("uciok");
            }
            UciMessage::IsReady => {
                println!("readyok");
            }
            UciMessage::UciNewGame => {
                board = Board::default();
                eng = Engine::new(tt_size_mb);
                eng.info = info;
                hist = History::new();
            }
            UciMessage::SetOption { name, value } => {
                if let Some(value) = value {
                    match name.as_str() {
                        "Hash" => tt_size_mb = value.parse().expect("parse"),
                        "Info" => info = value.parse().expect("parse"),
                        _ => eprintln!("> Invalid name!"),
                    }
                } else {
                    eprintln!("> No value recieved!")
                }

                // Reset engine
                eng = Engine::new(tt_size_mb);
                eng.info = info;
                hist = History::new();
            }
            UciMessage::Position {
                startpos,
                fen,
                moves,
            } => {
                if startpos {
                    board = Board::default();
                }

                if let Some(fen) = fen {
                    board = Board::from_str(fen.as_str()).expect("Parse fen");
                    hist.push_hist(board.get_hash());
                }

                for mv in moves {
                    board = board.make_move_new(mv);
                    hist.push_hist(board.get_hash());
                }
            }
            UciMessage::Go {
                time_control: tc,
                search_control: _,
            } => {
                let color = board.side_to_move();

                let tc = if let Some(tc) = tc {
                    match tc {
                        vampirc_uci::UciTimeControl::MoveTime(ms) => TimeManager {
                            max_depth: None,
                            max_nodes: None,
                            board_time: None,
                            max_allowed_time_now: Some(ms.num_milliseconds() as u32),
                        },
                        vampirc_uci::UciTimeControl::TimeLeft {
                            white_time,
                            black_time,
                            ..
                        } => TimeManager {
                            max_depth: None,
                            max_nodes: None,
                            max_allowed_time_now: None,
                            board_time: {
                                let w = white_time
                                    .map_or(60000, |white_time| white_time.num_milliseconds());
                                let b = black_time
                                    .map_or(60000, |black_time| black_time.num_milliseconds());

                                match color {
                                    chess::Color::Black => Some(b as u32),
                                    chess::Color::White => Some(w as u32),
                                }
                            },
                        },
                        _ => TimeManager {
                            max_depth: None,
                            max_nodes: None,
                            board_time: None,
                            max_allowed_time_now: None,
                        },
                    }
                } else {
                    TimeManager {
                        max_depth: None,
                        max_nodes: None,
                        board_time: Some(300000), // 5 minutes
                        max_allowed_time_now: Some(5000),
                    }
                };

                let mv = eng.start(board, tc, hist);
                println!("bestmove {mv}");
            }
            UciMessage::Quit => {
                return;
            }
            UciMessage::Unknown(str, _) => {
                eprintln!("> Could not parse message `{str}`!");
            }
            _ => {
                eprintln!("> Unimplemented message `{msg}`!");
            }
        }
    }
}
