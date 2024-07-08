use chess::Board;

use chess::Color::{Black, White};
use chess::Piece;

use crate::tables::{FLIP, PESTO};

const BISHOP_PAIR_BONUS: i32 = 50;

/// Helper function to calculate the game phase.
fn calculate_game_phase(board: &Board) -> i32 {
    let mut phase = 24;

    // Predefined values for phase decrement based on remaining pieces.
    let piece_phase_values = [
        (Piece::Pawn, 0), // Pawns do not affect the phase incrementally
        (Piece::Knight, 1),
        (Piece::Bishop, 1),
        (Piece::Rook, 2),
        (Piece::Queen, 4),
    ];

    for (piece, value) in piece_phase_values.iter() {
        let white_pieces = (board.pieces(*piece) & board.color_combined(White)).popcnt();
        let black_pieces = (board.pieces(*piece) & board.color_combined(Black)).popcnt();

        phase -= value * (white_pieces + black_pieces) as i32;
    }

    phase.clamp(0, 24)
}

/// Evaluation function.
pub fn eval(board: &Board) -> i32 {
    let mut mg_sc: i32 = 0; // Midgame score
    let mut eg_sc: i32 = 0; // Endgame score

    debug_assert!(board.is_sane());

    // Calculate the game phase
    let total_phase = calculate_game_phase(board);
    let mg_weight = total_phase;
    let eg_weight = 24 - total_phase;

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
                    Black => FLIP[sq.to_index()],
                };

                mg_sc += PESTO[sq_i][piece.to_index()][0] * color_mul;
                eg_sc += PESTO[sq_i][piece.to_index()][1] * color_mul;
            }
        }
    }

    // Bishop pair bonuses
    if (board.pieces(Piece::Bishop) & board.color_combined(White)).popcnt() >= 2 {
        mg_sc += BISHOP_PAIR_BONUS;
        eg_sc += BISHOP_PAIR_BONUS;
    }
    if (board.pieces(Piece::Bishop) & board.color_combined(Black)).popcnt() >= 2 {
        mg_sc -= BISHOP_PAIR_BONUS;
        eg_sc -= BISHOP_PAIR_BONUS;
    }

    let who2move = match board.side_to_move() {
        White => 1,
        Black => -1,
    };

    // Tapered score
    let sc = (mg_sc * mg_weight + eg_sc * eg_weight) / 24;

    sc * who2move
}

#[test]
fn sanity_check() {
    use std::str::FromStr;

    assert!(eval(&Board::from_str("1qkq4/2q5/8/8/8/8/5PPP/7K w - - 0 1").unwrap()) < -2000);
    assert!(eval(&Board::from_str("k7/ppp5/8/8/8/8/5Q2/4QKQ1 w - - 0 1").unwrap()) > 2000);
}
