use std::ops::Mul;

use crate::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct Phase(f32);

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct Earlygame(pub f32);

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct Endgame(pub f32);

impl Phase {
    #[must_use]
    #[inline]
    pub fn earlygame(self) -> Earlygame {
        Earlygame(self.0)
    }
    #[inline]
    #[must_use]
    pub fn endgame(self) -> Endgame {
        Endgame(1.0 - self.0)
    }
}

impl Mul<i32> for Earlygame {
    type Output = i32;
    #[inline]
    fn mul(self, rhs: i32) -> Self::Output {
        (self.0 * rhs as f32) as i32
    }
}

impl Mul<Earlygame> for i32 {
    type Output = i32;
    #[inline]
    fn mul(self, rhs: Earlygame) -> Self::Output {
        (rhs.0 * self as f32) as i32
    }
}

impl Mul<i32> for Endgame {
    type Output = i32;
    #[inline]
    fn mul(self, rhs: i32) -> Self::Output {
        (self.0 * rhs as f32) as i32
    }
}

impl Mul<Endgame> for i32 {
    type Output = i32;
    #[inline]
    fn mul(self, rhs: Endgame) -> Self::Output {
        (rhs.0 * self as f32) as i32
    }
}

impl Engine {
    #[inline]
    #[must_use]
    pub fn phase(&self) -> Phase {
        phase(&self.board)
    }
}

#[must_use]
#[inline]
pub fn phase(board: &Board) -> Phase {
    let mut sum = -6;
    sum += (board[Bishop] | board[Knight]).count() as i32;
    sum += 2 * board[Rook].count() as i32;
    sum += 4 * board[Queen].count() as i32;
    Phase((sum as f32 / 18.0).clamp(0.0, 1.0))
}
#[test]
#[allow(clippy::float_cmp)]
fn test_phase() {
    assert_eq!(phase(&Board::start_pos()).0, 1.0);
    assert_eq!(phase(&Board::from_fen("4k3/4p1n1/p5pp/1p3p2/8/5P2/1QP3PP/4K3 w - -").unwrap()).0, 0.0);
    assert_eq!(phase(&Board::from_fen("4k3/4p3/p1pp2pp/1p3p2/8/5P2/2PPP1PP/4K3 w - -").unwrap()).0, 0.0);
}
