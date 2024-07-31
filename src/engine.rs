use std::time::Instant;

use chess::{Board, BoardStatus, ChessMove};

use crate::{
    bump,
    eval::eval,
    stats::{
        self, add_move_index, CHECK_EXTENSION, NODES_SEARCHED, QNODES_SEARCHED, TT_CHECK, TT_HIT,
    },
    time::TimeManager,
    tt::{NodeType, TranspositionEntry, TT},
    utils::{is_mate_score, log_search_statistics, History, MoveGenOrdered, SearchInfo},
};

pub const OO: i32 = 10000;
/// Maximum number of moves
pub const MAX_PLY: u8 = 200;

pub struct Engine {
    pub tt: TT,
    pub info: bool,
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            tt: TT::new_with_size_mb(128),
            info: false,
        }
    }
}

impl Engine {
    #[must_use] pub fn new(tt_size_mb: usize) -> Self {
        Self {
            tt: TT::new_with_size_mb(tt_size_mb),
            info: false,
        }
    }

    /// Start a new search
    pub fn start(&mut self, board: Board, time: &TimeManager, history: History) -> ChessMove {
        stats::reset();
        let start_of_search_instant = Instant::now();

        let mut best_mv = None;

        let mut sinfo = SearchInfo::new();

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
            let best_score = self.negamax(&board, -OO, OO, depth, 0, &mut sinfo, history);

            best_mv = self.tt.get(board.get_hash()).best_move;

            if self.info {
                log_search_statistics(
                    depth,
                    best_score,
                    &start_of_search_instant,
                    &sinfo,
                    &board,
                    &self.tt,
                    best_mv,
                );
            }

            // return early if mate is found
            if is_mate_score(best_score) {
                return best_mv.unwrap();
            }
        }

        best_mv.expect("unable to find best move")
    }

    /// Starts a recursive negamax loop
    /// <https://www.chessprogramming.org/Negamax>
    /// <https://www.chessprogramming.org/Alpha-Beta>
    fn negamax(
        &mut self,
        board: &Board,
        mut alpha: i32, // minimum score that a node must reach in order to change the value of a previous node
        beta: i32,      // Beta is the best-score the opponent
        mut depth: u8,
        ply: u8,
        sinfo: &mut SearchInfo,
        history: History,
    ) -> i32 {
        bump!(NODES_SEARCHED);

        // Check extension
        let in_check = board.checkers().0 > 0;

        // QSearch to avoid Horizon effect
        // TODO Try not allowing qsearch if in check
        if (depth == 0 && !in_check) || ply > MAX_PLY {
            return self.qsearch(board, alpha, beta, sinfo, ply);
        }

        if history.is_three_rep() {
            return -OO;
        }

        // Check TT
        let key = board.get_hash();
        let old_alpha = alpha;
        let entry = self.tt.get(key);
        let mut tt_move = None;
        bump!(TT_CHECK);
        if entry.is_valid(key) && entry.depth >= depth {
            bump!(TT_HIT);

            // Correct entry scores
            let entry_value = if is_mate_score(entry.value) {
                if entry.value > 0 {
                    OO - (OO - entry.value) - ply as i32
                } else {
                    -OO + (entry.value + OO) + ply as i32
                }
            } else {
                entry.value
            };

            tt_move = entry.best_move;
            match entry.node_type {
                NodeType::Exact => return entry_value,
                NodeType::LowerBound => {
                    if entry_value >= beta {
                        return entry_value;
                    }
                }
                NodeType::UpperBound => {
                    if entry_value <= alpha {
                        return entry_value;
                    }
                }
                NodeType::Default => unreachable!(),
            }
        }

        let mut movegen = MoveGenOrdered::new(board, sinfo, ply, tt_move, false);
        match movegen.status() {
            chess::BoardStatus::Ongoing => {}
            chess::BoardStatus::Checkmate => return -OO + i32::from(ply),
            chess::BoardStatus::Stalemate => return 0,
        }

        // Check extention
        // https://www.chessprogramming.org/Check_Extensions
        // Also avoid flooding the stack by limiting it
        if in_check && ply < MAX_PLY / 2 {
            bump!(CHECK_EXTENSION);
            depth += 1;
        };

        let mut best_move = None;
        let mut best_score = -OO; // Initialize best_score to a very low value
                                  // Best move index to track location of best move, e.g. in 94% of cases the best move is first, etc.
        let mut best_move_index = 0;

        for mv_index in 0..movegen.len {
            let mv = movegen.pick_next().unwrap();
            let capture = board.piece_on(mv.get_dest()).is_some();

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

            if score > best_score {
                best_score = score;
                best_move = Some(mv);
                best_move_index = mv_index;

                if score > alpha {
                    alpha = score;
                    if !capture {
                        sinfo.history[mv.get_source().to_index()][mv.get_dest().to_index()] +=
                            u32::from(depth);
                    }
                }
            }

            if score >= beta {
                self.tt.set(TranspositionEntry {
                    key,
                    value: score,
                    depth,
                    node_type: NodeType::LowerBound,
                    best_move: Some(mv),
                });

                add_move_index(mv_index);

                if !capture {
                    sinfo.killers[1][ply as usize] = sinfo.killers[0][ply as usize];
                    sinfo.killers[0][ply as usize] = Some(mv);
                }

                return score;
            }
        }

        // Add move index to statistics
        add_move_index(best_move_index);

        // Add to TT
        self.tt.set(TranspositionEntry {
            key,
            value: alpha,
            depth,
            node_type: {
                if alpha > old_alpha {
                    NodeType::Exact
                } else {
                    NodeType::UpperBound
                }
            },
            best_move,
        });

        alpha
    }

    /// Quiescence Search
    /// <https://www.chessprogramming.org/Quiescence_Search>
    fn qsearch(
        &self,
        board: &Board,
        mut alpha: i32,
        beta: i32,
        sinfo: &SearchInfo,
        ply: u8,
    ) -> i32 {
        bump!(QNODES_SEARCHED);

        let standpat = eval(board);

        // Check if standpat causes a beta cutoff
        if standpat >= beta {
            return beta;
        }

        // Check if standpat may become a new alpha
        if standpat > alpha {
            alpha = standpat;
        }

        // TODO Add optional TT probing in qsearch
        // https://www.talkchess.com/forum/viewtopic.php?t=47373

        let mut movegen = MoveGenOrdered::new(board, sinfo, ply, None, true);
        match movegen.status() {
            BoardStatus::Ongoing => {}
            BoardStatus::Checkmate => return -OO + i32::from(ply),
            BoardStatus::Stalemate => return 0,
        }

        for _ in 0..movegen.len {
            let mv = movegen.pick_next().unwrap();
            let capture = board.piece_on(mv.get_dest()).is_some();
            debug_assert!(capture);

            let new_board = board.make_move_new(mv);
            let score = -self.qsearch(&new_board, -beta, -alpha, sinfo, ply + 1);

            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
            }
        }

        alpha
    }
}
