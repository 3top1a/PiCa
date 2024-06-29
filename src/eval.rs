use chess::Board;

use chess::Color::{Black, White};
use chess::Piece;

use crate::tables::PESTO;

const BISHOP_PAIR_BONUS: i32 = 50;
const ENDGAME_THR: u32 = 15;

/// Evaluation function.
pub fn eval(board: &Board) -> i32 {
    let mut sc: i32 = 0;

    let tempo = match board.combined().popcnt() {
        0..=ENDGAME_THR => 1,
        _ => 0,
    };

    for color in [White, Black] {
        let color_mul = if color == White { 1 } else { -1 };

        for piece in [
            Piece::Pawn,
            Piece::Knight,
            Piece::Bishop,
            Piece::Rook,
            Piece::Queen,
            Piece::King,
        ] {
            let pieces = board.pieces(piece) & board.color_combined(color);
            for sq in pieces {
                let sq_i = match color {
                    White => sq.to_index(),
                    Black => sq.to_index() ^ 56, // Flip index for black
                };

                sc += PESTO[sq_i][piece.to_index()][tempo as usize] * color_mul;
            }
        }
    }

    // https://www.chessprogramming.org/Bishop_Pair
    if (board.pieces(Piece::Bishop) & board.color_combined(White)).popcnt() >= 2 {
        sc += BISHOP_PAIR_BONUS;
    }
    if (board.pieces(Piece::Bishop) & board.color_combined(Black)).popcnt() >= 2 {
        sc -= BISHOP_PAIR_BONUS;
    }

    let who2move = match board.side_to_move() {
        White => 1,
        Black => -1,
    };

    sc * who2move
}
