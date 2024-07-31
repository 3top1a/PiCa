use std::time::Instant;

use chess::Board;
use vampirc_uci::UciTimeControl;

use crate::engine::MAX_PLY;

#[derive(Debug)]
#[derive(Default)]
pub struct TimeManager {
    pub max_depth: Option<u8>,
    pub _max_nodes: Option<u64>,
    pub board_time: Option<u32>,
    pub max_allowed_time_now: Option<u32>,
}


impl TimeManager {
    // https://www.chessprogramming.org/Time_Management
    #[must_use] pub fn can_continue(
        &self,
        depth: u8,
        _board: Board,
        _nodes: u64,
        start_of_search: Instant,
    ) -> bool {
        // TODO Yeet this HACK
        let estimate_time_branching_factor = 12;

        // Check for max depth
        if depth > self.max_depth.unwrap_or(MAX_PLY) {
            return false;
        }

        let time_ms = Instant::now();
        let ms = time_ms.duration_since(start_of_search).as_millis() as u32
            * estimate_time_branching_factor;
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

    #[must_use] pub fn from_uci(uci: &UciTimeControl, board: &Board) -> Self {
        match uci {
            UciTimeControl::Infinite => Self {
                max_depth: None,
                _max_nodes: None,
                board_time: None,
                max_allowed_time_now: None,
            },
            UciTimeControl::MoveTime(x) => Self {
                max_depth: None,
                _max_nodes: None,
                board_time: None,
                max_allowed_time_now: Some(x.num_milliseconds() as u32),
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
                    max_allowed_time_now: None,
                    max_depth: None,
                    _max_nodes: None,
                }
            }
        }
    }

    #[must_use] pub fn test_preset() -> Self {
        Self {
            board_time: None,
            max_allowed_time_now: Some(5000),
            max_depth: None,
            _max_nodes: None,
        }
    }
}
