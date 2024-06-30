use std::time::Instant;

use chess::{Board, ChessMove, MoveGen, Piece};

use crate::{
    bump,
    eval::eval,
    stats::{self, CHECK_EXTENSION, NODES_SEARCHED, TT_CHECK, TT_HIT},
    time::TimeManager,
    tt::{NodeType, TranspositionEntry, TT},
};

pub const OO: i32 = 10000;
pub const MAX_PLY: u8 = 200;
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

    let mvvlva = MVV_LVA[victim][attacker];

    mvvlva
}

fn sort_moves(a: ChessMove, b: ChessMove, board: &Board) -> core::cmp::Ordering {
    let a = score_move(a, board);
    let b = score_move(b, board);

    b.cmp(&a)
}

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

pub struct Engine {
    tt: TT,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            tt: TT::new_with_size_mb(256),
        }
    }

    /// Start a new search
    pub fn start(&mut self, board: Board, time: TimeManager) -> ChessMove {
        stats::reset();
        let start_of_search_instant = Instant::now();

        let mut alpha = -OO;
        let beta = OO;

        let mut best_mv = None;
        let mut best_score = -OO * 2;

        let mut sinfo = SearchInfo::new();

        for depth in 3..127 {
            if !time.can_continue(depth, board, start_of_search_instant) {
                break;
            }

            let movegen = MoveGen::new_legal(&board);
            let mut sorted_mv = movegen.collect::<Vec<ChessMove>>();
            sorted_mv.sort_by(|a, b| sort_moves(*a, *b, &board));

            for mv in sorted_mv {
                let nb = board.make_move_new(mv);
                sinfo.pv[0] = Some(mv);
                let score = -self.negamax(&nb, -beta, -alpha, depth, 1, &mut sinfo);

                if score > best_score {
                    best_score = score;
                    best_mv = Some(mv);
                }

                if score > alpha {
                    alpha = score;
                }

                if alpha >= beta {
                    break;
                }
            }

            unsafe {
                // Format: info score cp 20  depth 3 nodes 423 time 15 pv f1c4 g8f6 b1c3
                let time = Instant::now()
                    .duration_since(start_of_search_instant)
                    .as_millis();
                println!(
                    "info score cp {best_score} depth {depth} nodes {NODES_SEARCHED} time {} pv {} {}",
                    time,
                    best_mv.unwrap(),
                    sinfo.print()
                );
                println!(
                    "stats checkexts {} EBR {} TT Check {} hit {} nps {:.0}",
                    CHECK_EXTENSION,
                    (NODES_SEARCHED as f32).powf(1. / depth as f32),
                    TT_CHECK,
                    TT_HIT,
                    (1000 * NODES_SEARCHED as u128) / (time + 1)
                );
            }
        }

        best_mv.unwrap()
    }

    /// Starts a recursive negamax loop
    /// https://www.chessprogramming.org/Negamax
    /// https://www.chessprogramming.org/Alpha-Beta
    fn negamax(
        &mut self,
        board: &Board,
        mut alpha: i32,
        beta: i32,
        mut depth: u8,
        ply: u8,
        sinfo: &mut SearchInfo,
    ) -> i32 {
        bump!(NODES_SEARCHED);

        match board.status() {
            chess::BoardStatus::Ongoing => {}
            chess::BoardStatus::Checkmate => {
                return ply as i32 - OO;
            }
            chess::BoardStatus::Stalemate => {
                return 0;
            }
        }

        // Horizon
        // Also avoid stack overflow
        if depth == 0 || ply > MAX_PLY {
            return eval(board);
        }

        // Check TT
        let key = board.get_hash();
        let original_alpha = alpha;
        let entry = self.tt.get(key);
        bump!(TT_CHECK);
        if entry.is_valid(key) && entry.depth >= depth {
            bump!(TT_HIT);
            match entry.node_type {
                NodeType::Exact => return entry.value,
                NodeType::LowerBound => {
                    if entry.value >= beta {
                        return entry.value;
                    }
                }
                NodeType::UpperBound => {
                    if entry.value <= alpha {
                        return entry.value;
                    }
                }
                NodeType::Default => unreachable!(),
            }
        }

        // Check extension
        let in_check = board.checkers().0 > 0;
        if in_check {
            bump!(CHECK_EXTENSION);
            depth += 1
        };

        let movegen = MoveGen::new_legal(board);
        let mut sorted_mv = movegen.collect::<Vec<ChessMove>>();
        sorted_mv.sort_by(|a, b| sort_moves(*a, *b, board));
        if let Some(tt_move) = entry.best_move {
            if let Some(index) = sorted_mv.iter().position(|&m| m == tt_move) {
                sorted_mv.swap(0, index);
            }
        }

        let mut best_move = None;
        for mv in sorted_mv {
            // let capture = board.piece_on(mv.get_dest()).is_some();
            sinfo.pv[ply as usize] = Some(mv);

            let nb = board.make_move_new(mv);
            let score = -self.negamax(&nb, -beta, -alpha, depth - 1, ply + 1, sinfo);

            if score >= beta {
                self.tt.set(TranspositionEntry {
                    key,
                    value: beta,
                    depth,
                    node_type: NodeType::LowerBound,
                    best_move: Some(mv),
                });
                return beta;
            }
            if score > alpha {
                alpha = score;
                best_move = Some(mv);
            }
        }

        // Add to TT
        self.tt.set(TranspositionEntry {
            key,
            value: alpha,
            depth,
            node_type: {
                if alpha > original_alpha {
                    NodeType::Exact
                } else {
                    NodeType::UpperBound
                }
            },
            best_move,
        });

        alpha
    }
}
