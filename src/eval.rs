use cozy_chess::{BitBoard, Board, Color};

use cozy_chess::Color::{Black, White};
use cozy_chess::Piece;

use crate::tables::{EG, ISOLATED_PAWN_MASKS, MG, PASSED_PAWN_MASKS};

const PIECE_PHASE_VALUES: [i32; 6] = [0, 1, 1, 2, 4, 0];
const PASSED_PAWN_BONUS: [i32; 8] = [0, 0, 10, 30, 45, 70, 120, 200];
const ISOLATED_PAWN_PENALTY: i32 = -20;

/// Evaluation function.
pub fn eval(board: &Board) -> i32 {
    let who2move = match board.side_to_move() {
        White => 1,
        Black => -1,
    };

    let mut mg_sc: i32 = 0; // Midgame score
    let mut eg_sc: i32 = 0; // Endgame score

    // Calculate the game phase
    let mut game_phase = 0;

    // Get Pesto values
    for square in board.occupied() {
        if let Some(piece) = board.piece_on(square) {
            let color = unsafe { board.color_on(square).unwrap_unchecked() };

            game_phase += PIECE_PHASE_VALUES[piece as usize];
            let sq_i = square as usize;

            if color == Color::White {
                mg_sc += MG[piece as usize][sq_i];
                eg_sc += EG[piece as usize][sq_i];
            } else {
                mg_sc -= MG[piece as usize][sq_i ^ 56];
                eg_sc -= EG[piece as usize][sq_i ^ 56];
            }
        }
    }

    // TODO cache table
    for square in board.pieces(Piece::Pawn) {
        let color = board.color_on(square).unwrap();
        let color_mul = (color == White) as i32 * 2 - 1;
        let file = square.file() as usize;
        let rank = square.rank() as usize;

        let passed_mask = match color {
            White => PASSED_PAWN_MASKS[0][square as usize],
            Black => PASSED_PAWN_MASKS[1][square as usize],
        };

        let front_mask = match color {
            White => BitBoard(0x101010101010101 << square as usize),
            Black => BitBoard(0x101010101010101 >> (63 - square as usize)),
        } ^ BitBoard::from(square);

        if (board.colors(!color) & passed_mask == BitBoard::EMPTY)
            && (board.colors(color) & front_mask == BitBoard::EMPTY)
        {
            let bonus = if color == White {
                PASSED_PAWN_BONUS[rank]
            } else {
                PASSED_PAWN_BONUS[7 - rank]
            };
            mg_sc += color_mul * bonus;
            eg_sc += color_mul * bonus * 2;
        }

        // Isolated pawn evaluation
        let isolated_mask = ISOLATED_PAWN_MASKS[file];
        let friendly_pawns = board.pieces(Piece::Pawn) & board.colors(color);
        if friendly_pawns & isolated_mask == BitBoard::EMPTY {
            mg_sc += color_mul * ISOLATED_PAWN_PENALTY;
            eg_sc += color_mul * ISOLATED_PAWN_PENALTY / 2; // Less penalty in endgame
        }
    }

    // King distance
    /*let white_king_sq = board.king_square(White);
    let black_king_sq = board.king_square(Black);
    let a = white_king_sq
        .get_file()
        .to_index()
        .abs_diff(black_king_sq.get_file().to_index()) as i32;
    let b = white_king_sq
        .get_rank()
        .to_index()
        .abs_diff(black_king_sq.get_rank().to_index()) as i32;
    let d = a * a + b * b;
    eg_sc -= d;
    mg_sc += d;*/

    // Tempo bonus I guess
    // From https://www.chessprogramming.org/Tempo:
    // > That bonus is useful mainly in the opening and middle game positions, but can be counterproductive in the endgame.
    mg_sc += 10 * who2move;

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

    // Test for passed pawn scores
    assert_eq!(
        // "{} - {}",
        eval(&Board::from_str("6k1/8/8/8/8/P7/8/6K1 w - - 0 1").unwrap()),
        198
    );
    assert_eq!(
        // "{} - {}",
        eval(&Board::from_str("6k1/8/8/8/P7/8/8/6K1 w - - 0 1").unwrap()),
        176
    );
    assert_eq!(
        // "{} - {}",
        eval(&Board::from_str("6k1/8/8/P7/8/8/8/6K1 w - - 0 1").unwrap()),
        187
    );
    assert_eq!(
        // "{} - {}",
        eval(&Board::from_str("6k1/8/P7/8/8/8/8/6K1 w - - 0 1").unwrap()),
        228
    );
    assert_eq!(
        // "{} - {}",
        eval(&Board::from_str("6k1/P7/8/8/8/8/8/6K1 w - - 0 1").unwrap()),
        337
    );
    assert_eq!(
        // "{} - {}",
        eval(&Board::from_str("Q7/6k1/8/8/8/8/8/6K1 w - - 0 1").unwrap()),
        915
    );
}
