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
    pub nodes_evaluated: u64,
    pub nodes_evaluated_for_heighest_depth: u64,
}

impl Engine {
    #[must_use]
    pub fn new(board: Board) -> Self {
        Self {
            board,
            time_started: Instant::now(),
            time_available: Duration::from_secs(4),
            depth_reached: 0,
            nodes_evaluated: 0,
            nodes_evaluated_for_heighest_depth: 0,
        }
    }

    #[must_use]
    pub(crate) fn is_cancelled(&mut self) -> bool {
        self.time_started.elapsed() >= self.time_available
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
