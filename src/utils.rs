use std::time::Instant;

use chess::{Board, ChessMove, Piece};

use crate::{
    engine::MAX_PLY,
    stats::{CHECK_EXTENSION, NODES_SEARCHED, TT_CHECK, TT_HIT},
};

#[derive(Debug)]
pub struct SearchInfo {
    pub pv: [Option<ChessMove>; MAX_PLY as usize],
}

impl SearchInfo {
    pub fn new() -> Self {
        Self {
            pv: [None; MAX_PLY as usize],
        }
    }

    pub fn print(&self) -> String {
        self.pv
            .iter()
            .filter_map(|x| *x)
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(" ")
    }
}

pub fn log_search_statistics(
    depth: u8,
    best_mv: Option<ChessMove>,
    best_score: i32,
    start: &Instant,
    sinfo: &SearchInfo,
) {
    unsafe {
        let time = Instant::now().duration_since(*start).as_millis();
        println!(
            "info score cp {best_score} depth {depth} nodes {NODES_SEARCHED} time {time} pv {best_mv:?} {}",
            sinfo.print()
        );
        println!(
            "stats checkexts {CHECK_EXTENSION} EBR {} TT Check {TT_CHECK} hit {TT_HIT} nps {:.0}",
            (NODES_SEARCHED as f32).powf(1. / depth as f32),
            (1000 * NODES_SEARCHED as u128) / (time + 1)
        );
    }
}

pub const MVV_LVA: [[u8; chess::NUM_PIECES + 1]; chess::NUM_PIECES + 1] = [
    [0, 0, 0, 0, 0, 0, 0],
    [0, 15, 14, 13, 12, 11, 10],
    [0, 25, 24, 23, 22, 21, 20],
    [0, 35, 34, 33, 32, 31, 30],
    [0, 45, 44, 43, 42, 41, 40],
    [0, 55, 54, 53, 52, 51, 50],
    [0, 0, 0, 0, 0, 0, 0],
];

// TODO Remove this function and reorded table
fn piece_to_index(a: Option<Piece>) -> usize {
    match a {
        None => 0,
        Some(a) => a.to_index() + 1,
    }
}

fn score_move(mv: ChessMove, b: &Board) -> u8 {
    let attacker = piece_to_index(b.piece_on(mv.get_source()));
    let victim = piece_to_index(b.piece_on(mv.get_dest()));

    MVV_LVA[victim][attacker]
}

pub fn sort_moves(a: ChessMove, b: ChessMove, board: &Board) -> core::cmp::Ordering {
    let a = score_move(a, board);
    let b = score_move(b, board);

    b.cmp(&a)
}
