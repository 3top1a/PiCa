mod engine;
mod eval;
mod stats;
mod tables;
mod tests;
mod time;
mod tt;
mod utils;

use std::env::args;
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
    let mut info = true;

    let mut board = Board::default();
    let mut eng = Engine {
        info,
        ..Default::default()
    };
    let mut hist = History::new();

    // Check if args contain `--bench` and if so, search do a depth of 8
    if args().any(|x| x.contains("--bench")) {
        eng.start(
            Board::default(),
            &TimeManager {
                max_depth: Some(8),
                ..Default::default()
            },
            hist,
        );
        return;
    }

    for line in io::stdin().lock().lines() {
        let line = line.expect("receive stdin");

        // Print move index dist
        if line.trim() == "dist" {
            let x = unsafe { stats::MOVE_INDEX_DIST };
            let sum: u32 = x.iter().sum();
            for (i, x) in x.iter().enumerate() {
                if *x == 0 && i != 0 {
                    continue;
                }
                println!("{i}: {:.3}% ({x}/{sum})", (*x as f32 / sum as f32) * 100.);
            }
            continue;
        }

        let msg: UciMessage = parse_one(&line);
        match msg {
            UciMessage::Uci => {
                println!("id name PiCa v{}", env!("CARGO_PKG_VERSION"));
                println!("id author Filip Rusz <filip@rusz.space>");

                // List options
                println!("option name Hash type spin default 256 min 1 max 8192");
                println!("option name Info type check default true");

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
                    eprintln!("> No value recieved!");
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
                time_control,
                search_control: _,
            } => {
                let tc = match time_control {
                    Some(x) => TimeManager::from_uci(&x, &board),
                    None => TimeManager{max_allowed_time_now: Some(2500), ..Default::default()},
                };

                let mv = eng.start(board, &tc, hist);
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
