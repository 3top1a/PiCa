use std::time::Instant;

use cozy_chess::Board;
use cozy_chess::Color;
use cozy_uci::command::UciGoParams;

use crate::engine::MAX_PLY;

#[derive(Debug)]
// TODO: Convert u32/u64 to Duration
pub struct TimeManager {
    pub max_depth: Option<u8>,
    pub max_nodes: Option<u64>,
    pub board_time: Option<u32>,
    pub max_allowed_time_now: Option<u32>,
}

impl TimeManager {
    // https://www.chessprogramming.org/Time_Management
    pub fn can_continue(
        &self,
        depth: u8,
        board: Board,
        nodes: u64,
        start_of_search: Instant,
    ) -> bool {
        // TODO Yeet this HACK
        let estimate_time_branching_factor = 10;

        // Check for max depth
        if depth > self.max_depth.unwrap_or(MAX_PLY) {
            return false;
        }

        let time_ms = Instant::now();
        let ms = time_ms.duration_since(start_of_search).as_millis() as u32
            * estimate_time_branching_factor;
        let board_time = self.board_time.unwrap_or(300000);

        // Normal board time
        if ms > board_time / 30 {
            return false;
        }

        // Max allowed time
        if let Some(max_allowed_time_now) = self.max_allowed_time_now {
            if ms > max_allowed_time_now {
                return false;
            }
        }

        true
    }

    pub fn from_uci(params: UciGoParams, board: &Board) -> Self {
        let color = board.side_to_move();

        let board_time = match color {
            Color::White => params.wtime,
            Color::Black => params.btime,
        };

        Self {
            max_nodes: params.nodes,
            max_depth: params.depth.map(|d| d as u8),
            max_allowed_time_now: params.movetime.map(|d| d.as_millis() as u32),
            board_time: board_time.map(|x| x.as_millis() as u32),
        }
    }

    pub fn test_preset() -> Self {
        Self {
            board_time: None,
            max_allowed_time_now: Some(5000),
            max_depth: None,
            max_nodes: None,
        }
    }
}
