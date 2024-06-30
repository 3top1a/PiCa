mod engine;
mod eval;
mod stats;
mod tables;
mod tests;
mod tt;

use std::io;
use std::io::BufRead;
use std::str::FromStr;

use chess::Board;
use engine::Engine;
use vampirc_uci::parse_one;
use vampirc_uci::UciMessage;

fn main() {
    println!("PiCa v{}", env!("CARGO_PKG_VERSION"));

    let mut board = Board::default();
    let mut eng = Engine::new();

    for line in io::stdin().lock().lines() {
        let msg: UciMessage = parse_one(&line.expect("Parse UCI message"));
        match msg {
            UciMessage::Uci => {
                println!("id name BT7274");
                println!("id author Filip Rusz <filip@rusz.space>");

                // List options
                println!("option name Hash type spin default 256 min 1 max 2048");

                println!("uciok");
            }
            UciMessage::IsReady => {
                println!("readyok");
            }
            UciMessage::UciNewGame => {
                board = Board::default();
                eng = Engine::new();
            }
            UciMessage::SetOption { name, value } => {
                if let Some(value) = value {
                    match name.as_str() {
                        _ => println!("> Invalid name!"),
                    }
                } else {
                    println!("> No value recieved!")
                }

                // Reset engine
                eng = Engine::new();
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
                }

                for mv in moves {
                    board = board.make_move_new(mv);
                }
            }
            UciMessage::Go {
                time_control: _,
                search_control: _,
            } => {
                let mv = eng.start(board);
                println!("bestmove {mv}");
            }
            UciMessage::Quit => {
                return;
            }
            UciMessage::Unknown(str, _) => {
                println!("> Could not parse message `{str}`!");
            }
            _ => {
                println!("> Unimplemented message `{msg}`!");
            }
        }
    }
}
