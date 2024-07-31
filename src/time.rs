use std::time::Instant;

use chess::Board;
use vampirc_uci::UciTimeControl;

use crate::{engine::MAX_PLY, stats::NODES_SEARCHED};

#[derive(Debug)]
#[derive(Default)]
pub struct TimeManager {
    pub max_depth: Option<u8>,
    pub min_depth: Option<u8>,
    pub max_nodes: Option<u64>,
    pub board_time: Option<u32>,
    pub max_allowed_time_now: Option<u32>,
}

const ESTIMATE_TIME_BRANCHING_FACTOR: u32 = 8;

impl TimeManager {
    // https://www.chessprogramming.org/Time_Management
    #[must_use] pub fn can_continue_soft(
        &self,
        depth: u8,
        _board: Board,
        _nodes: u64,
        start_of_search: Instant,
    ) -> bool {
        // Check for minimum depth
        // Useful for CI/CD because the CPUs are slow
        if depth < self.min_depth.unwrap_or(0) {
            return true;
        }

        // Check for max depth
        if depth > self.max_depth.unwrap_or(MAX_PLY) {
            return false;
        }

        let time_ms = Instant::now();
        let ms = time_ms.duration_since(start_of_search).as_millis() as u32
            * ESTIMATE_TIME_BRANCHING_FACTOR;
        let board_time = self.board_time.unwrap_or(300_000);

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

    pub fn can_continue_hard(
        &self,
        depth: u8,
        _board: &Board,
        start_of_search: Instant,
    ) -> bool {
        // Check for max depth
        if depth > self.max_depth.unwrap_or(MAX_PLY) {
            // println!("fail hard max depth {}>{}", depth, self.max_depth.unwrap_or(MAX_PLY));
            return false;
        }

        let time_ms = Instant::now();
        let ms = time_ms.duration_since(start_of_search).as_millis() as u32;
        let board_time: u32 = self.board_time.unwrap_or(300_000);

        // Normal board time
        if ms > board_time / 20 {
            // println!("fail hard board time {} > {} / 20 ({})", ms, board_time, board_time / 20);
            return false;
        }

        // Nodes
        if unsafe { NODES_SEARCHED } > self.max_nodes.unwrap_or(u64::MAX) {
            // println!("fail hard nodes {} > {}", unsafe { NODES_SEARCHED }, self.max_nodes.unwrap_or(u64::MAX));
            return false;
        }

        // Max allowed time
        if let Some(max_allowed_time_now) = self.max_allowed_time_now {
            if ms > max_allowed_time_now {
                // println!("fail hard max allowed time {}>{}", ms, max_allowed_time_now);
                return false;
            }
        }

        true
    }

    #[must_use] pub fn from_uci(uci: &UciTimeControl, board: &Board) -> Self {
        match uci {
            UciTimeControl::Infinite => Self {
                ..Default::default()
            },
            UciTimeControl::MoveTime(x) => Self {
                max_allowed_time_now: Some(x.num_milliseconds() as u32),
                ..Default::default()
            },
            UciTimeControl::Ponder => todo!("podner not implemented yet"),
            UciTimeControl::TimeLeft {
                white_time,
                black_time,
                ..
            } => {
                let color = board.side_to_move();

                let time = match color {
                    chess::Color::White => white_time,
                    chess::Color::Black => black_time,
                };

                Self {
                    board_time: time.map(|x| x.num_milliseconds() as u32),
                    ..Default::default()
                }
            }
        }
    }

    #[must_use] pub fn test_preset() -> Self {
        Self {
            max_allowed_time_now: Some(5000),
            min_depth: Some(8),
            ..Default::default()
        }
    }
}
