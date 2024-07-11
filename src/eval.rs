use chess::{Board, Color, MoveGen, ALL_SQUARES, EMPTY};

use chess::Color::{Black, White};
use chess::Piece;

use crate::tables::{EG, FLIP, MG};

const PIECE_PHASE_VALUES: [i32; 6] = [0, 1, 1, 2, 4, 0];

/// Evaluation function.
pub fn eval(board: &Board) -> i32 {
    let mut mg_sc: i32 = 0; // Midgame score
    let mut eg_sc: i32 = 0; // Endgame score

    debug_assert!(board.is_sane());

    // Calculate the game phase
    let mut game_phase = 0;

    // Get Pesto values
    for square in *board.combined() {
        if let Some(piece) = board.piece_on(square) {
            let color = unsafe { board.color_on(square).unwrap_unchecked() };

            game_phase += PIECE_PHASE_VALUES[piece.to_index()];
            let sq_i = square.to_index();

            if color == Color::White {
                mg_sc += MG[piece.to_index()][sq_i];
                eg_sc += EG[piece.to_index()][sq_i];
            } else {
                mg_sc -= MG[piece.to_index()][sq_i ^ 56];
                eg_sc -= EG[piece.to_index()][sq_i ^ 56];
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
    let game_phase = game_phase.min(24);
    let mg_weight = game_phase;
    let eg_weight = 24 - game_phase;
    let sc = (mg_sc * mg_weight + eg_sc * eg_weight) / 24;

    sc * who2move
}

#[test]
fn sanity_check() {
    use std::str::FromStr;

    assert!(eval(&Board::from_str("1qkq4/2q5/8/8/8/8/5PPP/7K w - - 0 1").unwrap()) < -2000);
    assert!(eval(&Board::from_str("k7/ppp5/8/8/8/8/5Q2/4QKQ1 w - - 0 1").unwrap()) > 2000);
}
