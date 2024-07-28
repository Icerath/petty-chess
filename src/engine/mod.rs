mod evaluation;
mod move_ordering;
mod search;

use std::time::{Duration, Instant};

use crate::prelude::*;

pub struct Engine {
    pub board: Board,
    pub time_started: Instant,
    pub time_available: Duration,
    pub depth_reached: u8,
    pub total_nodes: u64,
    pub effective_nodes: u64,
    pub force_cancelled: bool,
}

impl Engine {
    #[must_use]
    pub fn new(board: Board) -> Self {
        Self {
            board,
            time_started: Instant::now(),
            time_available: Duration::from_secs(4),
            depth_reached: 0,
            total_nodes: 0,
            effective_nodes: 0,
            force_cancelled: false,
        }
    }

    #[must_use]
    pub(crate) fn is_cancelled(&mut self) -> bool {
        if self.force_cancelled {
            return true;
        }
        if self.time_started.elapsed() >= self.time_available {
            self.force_cancelled = true;
        }
        self.force_cancelled
    }
    pub fn endgame(&mut self) -> f32 {
        let num_start_pieces = 32;
        let mut num_pieces = 0;
        for pos in (0..64).map(Pos) {
            num_pieces += self.board[pos].is_some() as i32;
        }
        num_pieces as f32 / num_start_pieces as f32
    }
}
