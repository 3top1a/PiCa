use std::time::Instant;

use chess::Board;

pub struct TimeManager {
    pub max_depth: Option<u8>,
    pub max_nodes: Option<u64>,
    pub max_ms: Option<u32>,
    pub max_allowed_time: Option<u32>,
}

impl TimeManager {
    // https://www.chessprogramming.org/Time_Management
    pub fn can_continue(&self, depth: u8, board: Board, start_of_search: Instant) -> bool {
        // TODO Yeet this HACK
        let estimatebranchingfactor = 8;

        // Check for max depth
        if depth > self.max_depth.unwrap_or(255u8) {
            return false;
        }

        let pieces = (board.color_combined(chess::Color::White)
            | board.color_combined(chess::Color::Black))
        .popcnt() as u8;
        let time_ms = Instant::now();
        let diff = time_ms.duration_since(start_of_search).as_millis() as u32 * estimatebranchingfactor;

        if let Some(max) = self.max_ms {
            if diff > max {
                return false;
            }
        }

        // Check for max time
        // 1/30th the time 24..18 pieces
        // 1/20th the time after 17..10
        // 1/15th 9..0

        let divider = match pieces {
            18..=u8::MAX => 30,
            10..=17 => 20,
            0..=9 => 15,
        };

        let allowed_time = self.max_allowed_time.unwrap_or(300000) / divider;
        if diff > allowed_time {
            return false;
        }

        true
    }

    pub fn test_preset() -> Self {
        Self {
            max_ms: Some(2500),
            max_allowed_time: None,
            max_depth: None,
            max_nodes: None,
        }
    }
}
