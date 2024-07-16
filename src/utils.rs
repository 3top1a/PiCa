use std::{collections::HashSet, str::FromStr, time::Instant};

use chess::{Board, ChessMove, Piece};

use crate::{
    engine::MAX_PLY,
    stats::{CHECK_EXTENSION, NODES_SEARCHED, QNODES_SEARCHED, TT_CHECK, TT_HIT},
};

#[derive(Debug, Clone)]
pub struct SearchInfo {
    pub pv: [Option<ChessMove>; MAX_PLY as usize + 1],
    pub killers: [[Option<ChessMove>; MAX_PLY as usize + 1]; 2],
    pub history: [[u32; 64]; 64],
}

impl SearchInfo {
    pub fn new() -> Self {
        Self {
            pv: [None; MAX_PLY as usize + 1],
            killers: [[None; MAX_PLY as usize + 1]; 2],
            history: [[0; 64]; 64],
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
    best_score: i32,
    start: &Instant,
    sinfo: &SearchInfo,
    board: &Board,
) {
    /*let who2move = match board.side_to_move() {
        chess::Color::White => 1,
        chess::Color::Black => -1,
    };*/
    unsafe {
        let time = Instant::now().duration_since(*start).as_millis();
        println!(
            "info score cp {} depth {depth} nodes {NODES_SEARCHED} qnodes {QNODES_SEARCHED} time {time} pv {}",
            best_score,
            sinfo.print()
        );
        println!(
            "info string checkexts {CHECK_EXTENSION} EBR {} TT Check {TT_CHECK} hit {TT_HIT} nps {:.0}",
            (NODES_SEARCHED as f32).powf(1. / depth as f32),
            (1000 * NODES_SEARCHED as u128) / (time + 1)
        );
    }
}

pub const MVV_LVA: [[u32; chess::NUM_PIECES + 1]; chess::NUM_PIECES + 1] = [
    [0, 0, 0, 0, 0, 0, 0],
    [0, 1500, 1400, 1300, 1200, 1100, 1000],
    [0, 2500, 2400, 2300, 2200, 2100, 2000],
    [0, 3500, 3400, 3300, 3200, 3100, 3000],
    [0, 4500, 4400, 4300, 4200, 4100, 4000],
    [0, 5500, 5400, 5300, 5200, 5100, 5000],
    [0, 0, 0, 0, 0, 0, 0],
];

// TODO Remove this function and reorded table
fn piece_to_index(a: Option<Piece>) -> usize {
    return a.map_or(0, |a| a.to_index() + 1);
}

// TODO Tune this
pub static mut PV_VALUE: u32 = 30000;
pub static mut HASH_VALUE: u32 = 30000;
pub static mut KILLER_VALUE: u32 = 500;
pub static mut PROMOTIONS: [u32; chess::NUM_PIECES] = [0, 600, 700, 800, 900, 0];

fn score_move(
    mv: ChessMove,
    b: &Board,
    sinfo: &SearchInfo,
    ply: u8,
    hash: Option<ChessMove>,
) -> u32 {
    // Check if the move is in the PV
    if sinfo.pv[ply as usize] == Some(mv) {
        return unsafe { PV_VALUE };
    }

    // Check if move is best move indicated by TT
    if hash == Some(mv) {
        return unsafe { HASH_VALUE };
    }

    match mv.get_promotion() {
        Some(x) => return unsafe { PROMOTIONS[x.to_index()] },
        None => {}
    };

    let attacker = piece_to_index(b.piece_on(mv.get_source()));
    let victim = piece_to_index(b.piece_on(mv.get_dest()));

    let mvv_lva = MVV_LVA[victim][attacker];

    // If it's a capture, return MVV-LVA score
    if mvv_lva > 0 {
        return mvv_lva;
    }

    // Check if the move is a killer move
    if sinfo.killers[0][ply as usize] == Some(mv) {
        return unsafe { KILLER_VALUE };
    }
    if sinfo.killers[1][ply as usize] == Some(mv) {
        return unsafe { KILLER_VALUE } / 2;
    }

    // Otherwise, return the history score
    // Through testing it checks less nodes but is slower overall
    // sinfo.history[mv.get_source().to_index()][mv.get_dest().to_index()]

    mvv_lva
}

pub fn sort_moves(
    a: ChessMove,
    b: ChessMove,
    board: &Board,
    sinfo: &SearchInfo,
    ply: u8,
    hash: Option<ChessMove>,
) -> core::cmp::Ordering {
    let a = score_move(a, board, sinfo, ply, hash);
    let b = score_move(b, board, sinfo, ply, hash);

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
        return Self {
            history: [1, 2, 3, 4, 5, 6, 7, 8, 9],
        };
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
        let mut newhist = self.clone();
        newhist.history.copy_within(1.., 0);
        newhist.history[self.history.len() - 1] = new_key;
        return newhist;
    }

    pub fn is_three_rep(&self) -> bool {
        let newest = self.history[self.history.len() - 1];
        let mut reps = 0;

        reps += if newest == self.history[0] { 1 } else { 0 };
        reps += if newest == self.history[1] { 1 } else { 0 };
        reps += if newest == self.history[2] { 1 } else { 0 };
        reps += if newest == self.history[3] { 1 } else { 0 };
        reps += if newest == self.history[4] { 1 } else { 0 };
        reps += if newest == self.history[5] { 1 } else { 0 };
        reps += if newest == self.history[6] { 1 } else { 0 };
        reps += if newest == self.history[7] { 1 } else { 0 };
        reps += if newest == self.history[8] { 1 } else { 0 };

        reps >= 3
    }
}

mod tests {
    use crate::utils::History;
    use chess::{Board, ChessMove};
    use std::str::FromStr;

    #[test]
    // 1r4k1/pr1n3p/5np1/4p3/4P3/1P3PP1/5BB1/K1R3NR b - - 0 31
    fn test_three_rep() {
        let mut b = Board::from_str("8/8/k3K3/8/8/2Q5/8/8 w - - 5 9").unwrap();
        let mut h = History::new();
        h.push_hist(b.get_hash());

        for mvstr in [
            "Kd6", "Kb6", "Qb3+", "Ka5", "Kd5", "Ka6", "Qc2", "Ka5", "Qb3", "Ka6", "Qc2", "Ka5",
            "Qb3",
        ] {
            assert!(!h.is_three_rep());

            let mv = ChessMove::from_san(&b, mvstr).unwrap();
            b = b.make_move_new(mv);
            h.push_hist(b.get_hash());
        }

        assert!(h.is_three_rep());
    }

    // 1R6/5p2/8/1k1r4/3B4/P2PKP2/1P6/2R5 b - - 15 53
}
