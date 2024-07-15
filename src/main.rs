mod engine;
mod eval;
mod stats;
mod tables;
mod tests;
mod cache_table;
mod time;
mod tt;
mod utils;

use std::env::args;
use std::io::BufRead;
use std::io::{self, stdin};
use std::process::exit;
use std::str::FromStr;

use command::UciCommand;
use cozy_chess::Board;
use cozy_uci::UciParseErrorKind::UnknownMessageKind;
use cozy_uci::*;
use engine::Engine;
use remark::UciRemark;
use time::TimeManager;
use tt::TT;
use utils::{uncozy_move, History};

fn main() {
    // Incrementaly updated engine settings
    // TODO turn this into a struct
    let mut tt_size_mb = 256;
    let mut info = true;

    let mut board = Board::default();
    let mut eng = Engine {
        info,
        ..Default::default()
    };
    let mut hist = History::new();

    macro_rules! reset {
        () => {{
            eng = Engine {
                info,
                tt: TT::new_with_size_mb(tt_size_mb),
                ..Default::default()
            };
            hist = History::new();
            board = Board::default();
        }};
    }

    // If --prof is provided, inject `go` and `quit` commands
    let inject_prof_commands = args().any(|x| x.contains("--prof"));
    let mut injected_commands = Vec::new();
    if inject_prof_commands {
        injected_commands.push("go".to_string());
        injected_commands.push("quit".to_string());
    }
    let mut iter = injected_commands.into_iter();

    let options = UciFormatOptions::default();
    loop {
        let line = if let Some(injected_command) = iter.next() {
            injected_command
        } else {
            let mut input = String::new();
            if stdin().read_line(&mut input).is_err() {
                break;
            }
            input
        };

        let cmd = UciCommand::parse_from(&line, &options);

        if let Err(err) = cmd {
            eprintln!("{}", err);
            continue;
        }

        match cmd.unwrap() {
            UciCommand::Uci => {
                println!("id name PiCa v{}", env!("CARGO_PKG_VERSION"));
                println!("id author Filip Rusz <filip@rusz.space>");

                // List options
                println!("option name Hash type spin default 256 min 1 max 8192");
                println!("option name Info type check default true");

                println!("uciok");
            }
            UciCommand::IsReady => println!("readyok"),
            UciCommand::UciNewGame => reset!(),
            UciCommand::Quit => exit(0),
            UciCommand::SetOption { name, value } => {
                if let Some(value) = value {
                    match name.as_str() {
                        "Hash" => tt_size_mb = value.parse().expect("parse"),
                        "Info" => info = value.parse().expect("parse"),
                        _ => eprintln!("> Invalid option name!"),
                    }
                } else {
                    eprintln!("> No value recieved!")
                }

                // Reset engine
                reset!();
            }
            UciCommand::Position { init_pos, moves } => {
                match init_pos {
                    command::UciInitPos::Board(b) => board = b,
                    command::UciInitPos::StartPos => board = Board::startpos(),
                }
                for mv in moves {
                    board.play(uncozy_move(&board, mv));
                    hist.push_hist(board.hash());
                }
            }
            UciCommand::Go(params) => {
                let mv = eng.start(board.clone(), TimeManager::from_uci(params, &board), hist);
                println!("bestmove {mv}");
            }
            _ => eprintln!("{:?}", line),
        }
    }
}
