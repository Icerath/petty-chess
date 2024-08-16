pub mod evaluation;
mod move_ordering;
mod phase;
mod score;
mod search;
pub mod transposition;

use std::time::{Duration, Instant};

pub use phase::Phase;
use transposition::TranspositionTable;

use crate::prelude::*;

pub struct Engine {
    pub board: Board,
    pub seen_positions: Vec<Zobrist>,
    pub pv: Moves,
    pub depth_from_root: u16,
    pub time_started: Instant,
    pub time_available: Duration,
    pub depth_reached: u8,
    pub total_nodes: u64,
    pub effective_nodes: u64,
    pub force_cancelled: bool,
    pub transposition_table: TranspositionTable,
    pub only_pv_nodes: bool,
}

impl Engine {
    #[must_use]
    pub fn new(board: Board) -> Self {
        Self {
            board,
            pv: Moves::new(),
            depth_from_root: 0,
            seen_positions: vec![],
            time_started: Instant::now(),
            time_available: Duration::from_secs(4),
            depth_reached: 0,
            total_nodes: 0,
            effective_nodes: 0,
            force_cancelled: false,
            transposition_table: TranspositionTable::default(),
            only_pv_nodes: false,
        }
    }
    pub(crate) fn is_cancelled(&mut self) -> bool {
        self.time_started.elapsed() >= self.time_available || self.force_cancelled
    }
}
