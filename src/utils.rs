use std::{collections::HashSet, str::FromStr, time::Instant};

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
    board: &Board,
) {
    let who2move = match board.side_to_move() {
        chess::Color::White => 1,
        chess::Color::Black => -1,
    };
    if let Some(mv) = best_mv {
        unsafe {
            let time = Instant::now().duration_since(*start).as_millis();
            println!(
                "info score cp {} depth {depth} nodes {NODES_SEARCHED} time {time} pv {} {}",
                best_score,
                mv.to_string(),
                sinfo.print()
            );
            println!(
                "stats checkexts {CHECK_EXTENSION} EBR {} TT Check {TT_CHECK} hit {TT_HIT} nps {:.0}",
                (NODES_SEARCHED as f32).powf(1. / depth as f32),
                (1000 * NODES_SEARCHED as u128) / (time + 1)
            );
        }
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

/// Persistent data between games
#[derive(Debug, Clone, Copy)]
pub struct History {
    /// History using hashes
    history: [u64; 9],
}

impl History {
    pub fn new() -> Self {
        Self {
            history: [1, 2, 3, 4, 5, 6, 7, 8, 9],
        }
    }

    pub fn push_hist(&mut self, new_key: u64) {
        // Don't push if the newest entry is the same
        // This avoids a UCI bug where calling go multiple times fills up the history
        if self.history[self.history.len() - 1] == new_key {
            return;
        }

        self.history.copy_within(1.., 0);
        self.history[self.history.len() - 1] = new_key;
    }

    pub fn push_hist_new(&self, new_key: u64) -> Self {
        let mut l = self.clone();
        l.history.copy_within(1.., 0);
        l.history[self.history.len() - 1] = new_key;
        l
    }

    pub fn is_three_rep(&self) -> bool {
        self.history[0] == self.history[4] && self.history[0] == self.history[8]
    }
}

mod tests {
    use crate::utils::History;
    use chess::{Board, ChessMove};
    use std::str::FromStr;

    #[test]
    // 1r4k1/pr1n3p/5np1/4p3/4P3/1P3PP1/5BB1/K1R3NR b - - 0 31
    fn test_three_rep() {
        let mut b =
            Board::from_str("8/8/k3K3/8/8/2Q5/8/8 w - - 5 9").unwrap();
        let mut h = History::new();
        h.push_hist(b.get_hash());

        for mvstr in [
            "Kd6", "Kb6", "Qb3+", "Ka5", "Kd5", "Ka6", "Qc2", "Ka5", "Qb3", "Ka6", "Qc2", "Ka5", "Qb3",
        ] {
            assert!(!h.is_three_rep());

            let mv = ChessMove::from_san(&b, mvstr).unwrap();
            b = b.make_move_new(mv);
            h.push_hist(b.get_hash());
        }
        
        dbg!(h);
        assert!(h.is_three_rep());
    }

    // 1R6/5p2/8/1k1r4/3B4/P2PKP2/1P6/2R5 b - - 15 53
}
