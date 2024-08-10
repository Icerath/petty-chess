pub mod evaluation;
mod move_ordering;
mod score;
mod search;
pub mod transposition;

use std::time::{Duration, Instant};

use score::Eval;
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

    #[must_use]
    pub(crate) fn is_cancelled(&mut self) -> bool {
        self.time_started.elapsed() >= self.time_available || self.force_cancelled
    }
    #[must_use]
    #[inline]
    pub fn endgame(&self) -> f32 {
        endgame(&self.board)
    }
    #[inline]
    #[must_use]
    pub const fn infinity() -> i32 {
        Eval::INFINITY.0
    }
    #[inline]
    #[must_use]
    pub const fn mate_score() -> i32 {
        Eval::MATE_EVAL.0
    }
}

fn endgame(board: &Board) -> f32 {
    let mut sum = -6;
    sum += (board.get(Bishop) | board.get(Knight)).count() as i32;
    sum += 2 * board.get(Rook).count() as i32;
    sum += 4 * board.get(Queen).count() as i32;
    1.0 - (sum as f32 / 18.0).clamp(0.0, 1.0)
}

#[test]
#[allow(clippy::float_cmp)]
fn test_endgame() {
    assert_eq!(endgame(&Board::start_pos()), 0.0);
    assert_eq!(endgame(&Board::from_fen("4k3/4p1n1/p5pp/1p3p2/8/5P2/1QP3PP/4K3 w - -").unwrap()), 1.0);
    assert_eq!(endgame(&Board::from_fen("4k3/4p3/p1pp2pp/1p3p2/8/5P2/2PPP1PP/4K3 w - -").unwrap()), 1.0);
}
