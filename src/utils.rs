use std::time::Instant;

use arrayvec::ArrayVec;

use chess::{BitBoard, Board, BoardStatus, ChessMove, MoveGen, Piece};

use crate::{
    engine::{MAX_PLY, OO},
    stats::{CHECK_EXTENSION, NODES_SEARCHED, QNODES_SEARCHED, TT_CHECK, TT_HIT},
    tt::TT,
};

#[derive(Debug)]
pub struct SearchInfo {
    pub killers: [[Option<ChessMove>; MAX_PLY as usize + 1]; 2],
    pub history: [[u32; 64]; 64],
    pub start: Instant,
}

impl Default for SearchInfo {
    fn default() -> Self {
        Self::new(Instant::now())
    }
}

impl SearchInfo {
    #[must_use]
    pub fn new(start: Instant) -> Self {
        Self {
            killers: [[None; MAX_PLY as usize + 1]; 2],
            history: [[0; 64]; 64],
            start,
        }
    }
}

fn printpv(tt: &TT, board: &Board, current_best: Option<ChessMove>) -> String {
    let mut board = *board;
    let mut pv = Vec::with_capacity(64);
    if let Some(current_best) = current_best {
        pv.push(current_best);
        board = board.make_move_new(current_best);
    }

    let mut depth = 1;

    // Have a hard cap on PV because run away may happen
    for _ in 0..63 {
        let key = board.get_hash();
        let entry = tt.get(key);

        // dbg!("Depth: {}, Key: {:x}, Valid: {}, Entry: {}", depth, key, entry.is_valid(key), entry);
        if entry.is_valid(key) && entry.depth >= depth {
            if let Some(mv) = entry.best_move {
                // dbg!("  Found move: {}", mv);
                pv.push(mv);
                board = board.make_move_new(mv);
                depth += 1;
            } else {
                // dbg!("  No best move found");
                break;
            }
        } else {
            // dbg!("  Invalid entry or insufficient depth");
            break;
        }
    }

    // dbg!("Total PV length: {}", pv.len());
    pv.iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>()
        .join(" ")
}

pub const MAX_MOVES: usize = 128;

pub struct MoveGenOrdered {
    moves: ArrayVec<(ChessMove, i32), MAX_MOVES>,
    pub len: usize,
    real_len: usize,
    board_checkers: u64,
}

impl MoveGenOrdered {
    #[must_use]
    pub fn new(
        board: &Board,
        sinfo: &SearchInfo,
        ply: u8,
        tt_move: Option<ChessMove>,
        caponly: bool,
    ) -> Self {
        let mut moves = ArrayVec::new();

        let movegen = MoveGen::new_legal(board);

        // If it's capture only, make the target squares the opponents squares
        // otherwise it's all squares
        let targets = if caponly {
            *board.color_combined(!board.side_to_move())
        } else {
            !chess::EMPTY
        };

        // keep track of the real length without the target for status checking
        let mut real_len = 0;

        for mv in movegen {
            if (BitBoard::from_square(mv.get_dest()) & targets).0 != 0 {
                let score = score_move(mv, board, sinfo, ply, tt_move);
                moves.push((mv, score as i32));
            }

            real_len += 1;
        }

        Self {
            len: moves.len(),
            real_len,
            moves,
            board_checkers: board.checkers().0,
        }
    }

    #[must_use]
    pub fn status(&self) -> BoardStatus {
        match self.real_len {
            0 => {
                if self.board_checkers == 0 {
                    BoardStatus::Stalemate
                } else {
                    BoardStatus::Checkmate
                }
            }
            _ => BoardStatus::Ongoing,
        }
    }

    pub fn pick_next(&mut self) -> Option<ChessMove> {
        let mut best_mv = None;
        let mut best_score = -1;
        let mut best_index = 0;

        for mv_i in 0..self.moves.len() {
            let mv = self.moves[mv_i];
            if mv.1 > best_score {
                best_mv = Some(mv.0);
                best_score = mv.1;
                best_index = mv_i;
            }
        }

        self.moves.remove(best_index);

        best_mv
    }
}

pub fn log_search_statistics(
    depth: u8,
    best_score: i32,
    start: &Instant,
    _sinfo: &SearchInfo,
    board: &Board,
    tt: &TT,
    bestmv: Option<ChessMove>,
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
            printpv(tt, board, bestmv),
        );
        println!(
            "info string checkexts {CHECK_EXTENSION} EBR {} TT Check {TT_CHECK} hit {TT_HIT} nps {:.0}",
            (NODES_SEARCHED as f64).powf(1. / f64::from(depth)),
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
    a.map_or(0, |a| a.to_index() + 1)
}

// TODO Tune this
const HASH_VALUE: u32 = 50;
const KILLER_VALUE: u32 = 20;

fn score_move(
    mv: ChessMove,
    b: &Board,
    sinfo: &SearchInfo,
    ply: u8,
    hash: Option<ChessMove>,
) -> u32 {
    // Check if move is best move indicated by TT
    if hash == Some(mv) {
        return HASH_VALUE;
    }

    let attacker = piece_to_index(b.piece_on(mv.get_source()));
    let victim = piece_to_index(b.piece_on(mv.get_dest()));

    let mvv_lva = u32::from(MVV_LVA[victim][attacker]);

    // If it's a capture, return MVV-LVA score
    if mvv_lva > 0 {
        return mvv_lva;
    }

    // Check if the move is a killer move
    if sinfo.killers[0][ply as usize] == Some(mv) {
        return KILLER_VALUE;
    }
    if sinfo.killers[1][ply as usize] == Some(mv) {
        return KILLER_VALUE - 10;
    }

    // Otherwise, return the history score
    // Through testing it checks less nodes but is slower overall
    // TODO fix this
    // sinfo.history[mv.get_source().to_index()][mv.get_dest().to_index()]

    mvv_lva
}

// #[must_use] pub fn sort_moves(
//     a: ChessMove,
//     b: ChessMove,
//     board: &Board,
//     sinfo: &SearchInfo,
//     ply: u8,
//     hash: Option<ChessMove>,
// ) -> core::cmp::Ordering {
//     let a = score_move(a, board, sinfo, ply, hash);
//     let b = score_move(b, board, sinfo, ply, hash);

//     b.cmp(&a)
// }

const MATE_SCORE: i32 = OO - u8::MAX as i32;
pub const fn is_mate_score(score: i32) -> bool {
    score >= MATE_SCORE || score <= -MATE_SCORE
}

/// Persistent data between games
#[derive(Debug, Clone, Copy)]
pub struct History {
    /// History using hashes
    history: [u64; 9],
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

impl History {
    #[must_use]
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

    #[must_use]
    pub fn push_hist_new(&self, new_key: u64) -> Self {
        let mut newhist = *self;
        newhist.history.copy_within(1.., 0);
        newhist.history[self.history.len() - 1] = new_key;
        newhist
    }

    #[must_use]
    pub fn is_three_rep(&self) -> bool {
        let newest: u64 = self.history[self.history.len() - 1];
        let mut reps = 0u8;

        for i in 0..self.history.len() {
            reps += u8::from(newest == self.history[i]);
        }

        reps >= 3
    }
}

mod tests {
    #[test]
    // 1r4k1/pr1n3p/5np1/4p3/4P3/1P3PP1/5BB1/K1R3NR b - - 0 31
    // TODO more tests
    fn test_three_rep() {
        use crate::utils::History;
        use chess::{Board, ChessMove};
        use std::str::FromStr;

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
