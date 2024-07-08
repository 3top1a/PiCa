use std::time::Instant;

use chess::{Board, ChessMove, MoveGen};

use crate::{
    bump,
    eval::eval,
    stats::{self, CHECK_EXTENSION, NODES_SEARCHED, TT_CHECK, TT_HIT},
    time::TimeManager,
    tt::{NodeType, TranspositionEntry, TT},
    utils::{log_search_statistics, sort_moves, History, SearchInfo},
};

pub const OO: i32 = 10000;
/// Maximum number of moves
pub const MAX_PLY: u8 = 200;

pub struct Engine {
    tt: TT,
}

impl Engine {
    pub fn new(tt_size_mb: usize) -> Self {
        Self {
            tt: TT::new_with_size_mb(tt_size_mb),
        }
    }

    /// Start a new search
    pub fn start(&mut self, board: Board, time: TimeManager, history: History) -> ChessMove {
        stats::reset();
        let start_of_search_instant = Instant::now();

        let mut alpha = -OO;
        let beta = OO;

        let mut best_mv = None;
        let mut best_score = -OO * 2;

        let mut sinfo = SearchInfo::new();

        if history.is_three_rep() {
            println!("debug In a three repetition position, no moves possible");
            panic!();
        }

        println!("{:?}", time);

        // keep track of the number of nodes last ply, if it doesn't change with another iteration we are screwed anyways
        let mut nodes_last_ply = 0;
        for depth in 1..MAX_PLY {
            if !time.can_continue(
                depth,
                board,
                unsafe { NODES_SEARCHED },
                start_of_search_instant,
            ) {
                break;
            }

            stats::reset();

            let movegen = MoveGen::new_legal(&board);
            let mut sorted_mv = movegen.collect::<Vec<ChessMove>>();
            sorted_mv.sort_by(|a, b| sort_moves(*a, *b, &board));

            for mv in sorted_mv {
                if history.is_three_rep() {
                    break;
                }

                let nb = board.make_move_new(mv);
                sinfo.pv[0] = Some(mv);
                let new_history = history.push_hist_new(board.get_hash());
                let score = -self.negamax(&nb, -beta, -alpha, depth, 1, &mut sinfo, new_history);

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

            log_search_statistics(
                depth,
                best_mv,
                best_score,
                &start_of_search_instant,
                &sinfo,
                &board,
            );

            if nodes_last_ply == unsafe { NODES_SEARCHED } {
                println!(
                    "debug No change in searched nodes despite larger search depth, exiting early"
                );
                break;
            }
            nodes_last_ply = unsafe { NODES_SEARCHED };
        }

        best_mv.expect("unable to find best move")
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
        history: History,
    ) -> i32 {
        bump!(NODES_SEARCHED);

        match board.status() {
            chess::BoardStatus::Ongoing => {}
            chess::BoardStatus::Checkmate => return -OO + ply as i32,
            chess::BoardStatus::Stalemate => return 0,
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

        if history.is_three_rep() {
            return -OO;
        }

        // Check extension
        let in_check = board.checkers().0 > 0;
        // Also avoid flooding the stack
        if in_check && ply < 20 {
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

            let new_board = board.make_move_new(mv);
            let new_history: History = history.push_hist_new(new_board.get_hash());
            let score = -self.negamax(
                &new_board,
                -beta,
                -alpha,
                depth - 1,
                ply + 1,
                sinfo,
                new_history,
            );

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
