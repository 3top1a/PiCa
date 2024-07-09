use chess::{Board, MoveGen, EMPTY};

use chess::Color::{Black, White};
use chess::Piece;

use crate::tables::{EG, FLIP, MG};


const PIECE_PHASE_VALUES: [(Piece, i32); 5] = [
    (Piece::Pawn, 0),
    (Piece::Knight, 1),
    (Piece::Bishop, 1),
    (Piece::Rook, 2),
    (Piece::Queen, 4),
];

/// Helper function to calculate the game phase.
fn calculate_game_phase(board: &Board) -> i32 {
    let mut phase = 24;
    
    for &(piece, value) in PIECE_PHASE_VALUES.iter() {
        let pieces = board.pieces(piece).popcnt();
        phase -= value * (pieces) as i32;
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

    // Get Pesto values
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

                mg_sc += MG[piece.to_index()][sq_i] * color_mul;
                eg_sc += EG[piece.to_index()][sq_i] * color_mul;
            }
        }
    }

    // Tempo bonus I guess
    // From https://www.chessprogramming.org/Tempo:
    // > That bonus is useful mainly in the opening and middle game positions, but can be counterproductive in the endgame. 
    mg_sc += 10;

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
