use crate::utils::sort_moves;
use crate::utils::SearchInfo;
use chess::Board;
use chess::ChessMove;

pub static mut MOVE_INDEX_DIST: Vec<MoveOrd> = vec![];

#[derive(Debug, Clone)]
pub struct MoveOrd {
    pub sorted_mv: Vec<ChessMove>,
    pub board: Board,
    pub sinfo: SearchInfo,
    pub ply: u8,
    pub tt_move: Option<ChessMove>,
    pub best_move_index: usize,
}

#[cfg(not(feature = "moveortune"))]
#[inline(always)]
pub fn move_ord_tune(
    sorted_mv: &Vec<ChessMove>,
    board: &Board,
    sinfo: &SearchInfo,
    ply: u8,
    tt_move: Option<ChessMove>,
    best_move_index: usize,
) {
    // do nothing
}

#[cfg(feature = "moveortune")]
pub fn move_ord_tune(
    sorted_mv: &Vec<ChessMove>,
    board: &Board,
    sinfo: &SearchInfo,
    ply: u8,
    tt_move: Option<ChessMove>,
    best_move_index: usize,
) {
    unsafe {
        MOVE_INDEX_DIST.push(MoveOrd {
            best_move_index,
            board: *board,
            ply,
            sinfo: sinfo.clone(),
            sorted_mv: sorted_mv.to_vec(),
            tt_move,
        })
    }
}

pub fn score(move_ords: Vec<MoveOrd>) -> f32 {
    let mut move_ords = move_ords.clone();
    for mv in &mut move_ords {
        let best_mv = mv.sorted_mv[mv.best_move_index];
        mv.sorted_mv
            .sort_by(|a, b| sort_moves(*a, *b, &mv.board, &mv.sinfo, mv.ply, mv.tt_move));
        mv.best_move_index = mv.sorted_mv.iter().position(|x| *x == best_mv).unwrap();
    }

    let a = move_ords
        .iter()
        .map(|x| x.best_move_index as f32 + 1.)
        .sum::<f32>()
        / move_ords.len() as f32;
    let b = move_ords.iter().map(|x| x.best_move_index).max().unwrap() as f32;
    let c = move_ords
        .iter()
        .map(|x| x.best_move_index as f32)
        .sum::<f32>();

    return a;
}
