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
    pub time_started: Instant,
    pub time_available: Duration,
    pub depth_reached: u8,
    pub total_nodes: u64,
    pub effective_nodes: u64,
    pub force_cancelled: bool,
    pub transposition_table: TranspositionTable,
}

impl Engine {
    #[must_use]
    pub fn new(board: Board) -> Self {
        Self {
            board,
            pv: Moves::new(),
            seen_positions: vec![],
            time_started: Instant::now(),
            time_available: Duration::from_secs(4),
            depth_reached: 0,
            total_nodes: 0,
            effective_nodes: 0,
            force_cancelled: false,
            transposition_table: TranspositionTable::default(),
        }
    }

    #[must_use]
    pub(crate) fn is_cancelled(&mut self) -> bool {
        self.time_started.elapsed() >= self.time_available || self.force_cancelled
    }
    pub fn endgame(&mut self) -> f32 {
        let default_sum = 24;
        let mut sum = 0;
        let mut num_queens = [0; 2];
        for piece in self.board.pieces() {
            sum += match piece.kind() {
                PieceKind::Pawn | PieceKind::King => 0,
                PieceKind::Knight | PieceKind::Bishop => 1,
                PieceKind::Rook => 2,
                PieceKind::Queen => 4,
            };
            if piece.kind() == Queen {
                num_queens[piece.colour() as usize] += 1;
            }
        }
        let sum = sum.min(default_sum);
        1.0 - (sum as f32 / default_sum as f32)
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
