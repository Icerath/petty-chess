pub mod evaluation;
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
    pub fn infinity(&self) -> i32 {
        i32::MAX
    }
}
