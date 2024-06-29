use chess::{BitBoard, Board, CacheTable, ChessMove, File, MoveGen, Piece, Rank};

use crate::{
    bump,
    eval::eval,
    stats::{self, dump, CHECK_EXTENSION, NODES_SEARCHED},
};

const OO: i32 = 10000;

#[derive(Clone, Debug)]
pub struct Engine {}

impl Engine {
    pub fn new() -> Self {
        Self {}
    }

    /// Start a new search
    pub fn start(&mut self, board: Board) -> ChessMove {
        stats::reset();

        let mut alpha = -OO;
        let beta = OO;
        let movegen = MoveGen::new_legal(&board);

        let mut best_mv = None;
        let mut best_score = -OO * 2;

        for mv in movegen {
            let nb = board.make_move_new(mv);
            let score = -self.negamax(&nb, -beta, -alpha, 3, 1);

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

        // println!(
        //     "info score cp {best_score} depth {} nodes {} leafs {} time {} pv {}",
        //     depth + 1,
        //     self.s.searched_nodes,
        //     self.s.qsearched_leafs,
        //     elapsed_millis(start_time),
        //     best_mv.unwrap(),
        // );

        dump(3);

        best_mv.unwrap()
    }

    /// Starts a recursive negamax loop
    /// https://www.chessprogramming.org/Negamax
    /// https://www.chessprogramming.org/Alpha-Beta
    fn negamax(&mut self, board: &Board, mut alpha: i32, beta: i32, mut depth: u8, ply: u8) -> i32 {
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
        if depth == 0 || ply > 100 {
            return eval(board);
        }

        // Check extension
        let in_check = board.checkers().0 > 0;
        if in_check {
            bump!(CHECK_EXTENSION);
            depth += 1
        };

        let mut movegen = MoveGen::new_legal(board);

        // let mut sorted_mv = movegen.collect::<Vec<ChessMove>>();
        // sorted_mv.sort_by(|a, b| sort_moves(*a, *b, board));

        for mv in &mut movegen {
            // let capture = board.piece_on(mv.get_dest()).is_some();

            let nb = board.make_move_new(mv);
            let score = -self.negamax(&nb, -beta, -alpha, depth - 1, ply + 1);

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
