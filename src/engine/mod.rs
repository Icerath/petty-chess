pub mod evaluation;
mod mobility;
mod move_ordering;
mod phase;
mod score;
mod search;
pub mod transposition;

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Instant,
};

pub use phase::Phase;
use transposition::TranspositionTable;

use crate::prelude::*;

#[derive(Clone)]
pub struct Engine {
    pub board: Board,
    pub seen_positions: Vec<Zobrist>,
    pub kill: Arc<AtomicBool>,
    pub pv: Moves,
    pub depth_from_root: u16,
    pub time_started: Instant,
    pub depth_reached: u8,
    pub total_nodes: u64,
    pub effective_nodes: u64,
    pub transposition_table: TranspositionTable,
    pub only_pv_nodes: bool,
}

impl Engine {
    #[must_use]
    pub fn new(board: Board) -> Self {
        Self {
            kill: Arc::default(),
            board,
            pv: Moves::new(),
            depth_from_root: 0,
            seen_positions: vec![],
            time_started: Instant::now(),
            depth_reached: 0,
            total_nodes: 0,
            effective_nodes: 0,
            transposition_table: TranspositionTable::default(),
            only_pv_nodes: false,
        }
    }

    pub(crate) fn is_cancelled(&mut self) -> bool {
        self.kill.load(Ordering::Relaxed)
    }
}
