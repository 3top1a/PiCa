use std::time::Instant;

use chess::Board;

use crate::engine::MAX_PLY;

#[derive(Debug)]
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

    pub fn test_preset() -> Self {
        Self {
            board_time: Some(2500),
            max_allowed_time_now: None,
            max_depth: None,
            max_nodes: None,
        }
    }
}
