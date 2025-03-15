pub mod evaluation;
mod mobility;
mod move_ordering;
mod phase;
mod score;
mod search;
pub mod transposition;

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

pub use phase::{Phase, phase};
pub use score::Eval;
use transposition::TranspositionTable;

use crate::prelude::*;

#[derive(Clone)]
pub struct Engine {
    pub board: Board,
    pub seen_positions: Vec<Zobrist>,
    pub time_available: Duration,
    pub kill: Arc<AtomicBool>,
    pub pv: Vec<Move>,
    pub depth_from_root: u16,
    pub total_nodes: u64,
    pub transposition_table: TranspositionTable<()>,
    pub only_pv_nodes: bool,
}

impl Engine {
    #[must_use]
    pub fn new(board: Board) -> Self {
        Self {
            kill: Arc::default(),
            time_available: Duration::MAX,
            board,
            pv: vec![],
            depth_from_root: 0,
            seen_positions: vec![],
            total_nodes: 0,
            transposition_table: TranspositionTable::default(),
            only_pv_nodes: false,
        }
    }

    pub(crate) fn is_cancelled(&self) -> bool {
        self.kill.load(Ordering::Relaxed)
    }
}
