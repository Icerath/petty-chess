mod evaluation;
mod search;

use std::time::{Duration, Instant};

use crate::prelude::*;

pub struct Engine {
    pub board: Board,
    pub time_started: Instant,
    pub time_available: Duration,
}

impl Engine {
    #[must_use]
    pub fn new(board: Board) -> Self {
        Self { board, time_started: Instant::now(), time_available: Duration::from_secs(1) }
    }

    #[must_use]
    pub(crate) fn is_cancelled(&mut self) -> bool {
        // is_cancelled should be called roughly once per position evaluated
        self.time_started.elapsed() >= self.time_available
    }
}
