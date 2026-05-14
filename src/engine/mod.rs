pub mod evaluation;
mod mobility;
mod move_ordering;
mod phase;
pub mod psqt;
mod score;
mod search;
pub mod transposition;

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

pub use phase::{Phase, phase};
pub use score::Score;
use transposition::TranspositionTable;

use crate::prelude::*;

#[derive(Clone)]
pub struct Engine {
    pub board: Board,
    pub seen_positions: Vec<Zobrist>,
    pub time_started: Instant,
    pub time_available: Duration,
    pub kill: Arc<AtomicBool>,
    pub pv: Vec<Move>,
    pub depth_from_root: u16,
    pub total_nodes: u64,
    pub transposition_table: TranspositionTable<Option<Move>>,
    pub killer: Vec<Option<Move>>,
    pub cancel_check: u32,
}

impl Engine {
    #[must_use]
    pub fn new(board: Board) -> Self {
        Self {
            kill: Arc::default(),
            time_started: Instant::now(),
            time_available: Duration::MAX,
            board,
            pv: vec![],
            depth_from_root: 0,
            seen_positions: vec![],
            total_nodes: 0,
            transposition_table: TranspositionTable::from_mb(0),
            killer: vec![],
            cancel_check: 0,
        }
    }

    pub(crate) fn is_cancelled(&mut self) -> bool {
        if self.cancel_check < 1024 {
            self.cancel_check += 1;
            return false;
        }
        std::hint::cold_path();
        if self.kill.load(Ordering::Relaxed) || self.time_started.elapsed() > self.time_available {
            true
        } else {
            self.cancel_check = 0;
            false
        }
    }
}
