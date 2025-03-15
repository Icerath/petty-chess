use std::ops::Mul;

use crate::prelude::*;

const UPPER: i32 = 32;
const UPPER_FLOAT: f32 = UPPER as f32;

#[derive(Debug, Clone, Copy)]
pub struct Phase(i32);

impl Phase {
    #[must_use]
    pub const fn earlygame(self) -> Earlygame {
        Earlygame(self.0)
    }

    #[must_use]
    pub const fn endgame(self) -> Endgame {
        Endgame(UPPER - self.0)
    }
}

#[must_use]
pub fn phase(board: &Board) -> Phase {
    let mut sum = -6;
    sum += (board[Bishop] | board[Knight]).count() as i32;
    sum += 2 * board[Rook].count() as i32;
    sum += 4 * board[Queen].count() as i32;
    Phase(((sum * UPPER) / 18).clamp(0, UPPER))
}

macro_rules! impl_stage {
    ($stage: ident) => {
        #[derive(Debug, PartialEq, Eq, PartialOrd, Clone, Copy)]
        pub struct $stage(i32);

        impl $stage {
            pub fn as_float(self) -> f32 {
                self.0 as f32 / UPPER_FLOAT
            }
        }

        impl Mul<i32> for $stage {
            type Output = i32;

            fn mul(self, rhs: i32) -> Self::Output {
                (self.0 * rhs) / UPPER
            }
        }

        impl Mul<$stage> for i32 {
            type Output = i32;

            fn mul(self, rhs: $stage) -> Self::Output {
                (self * rhs.0) / UPPER
            }
        }
    };
}

impl_stage!(Earlygame);
impl_stage!(Endgame);

#[test]
fn test_phase() {
    assert_eq!(phase(&Board::start_pos()).0, UPPER);
    assert_eq!(
        phase(&Board::from_fen("4k3/4p1n1/p5pp/1p3p2/8/5P2/1QP3PP/4K3 w - -").unwrap()).0,
        0
    );
    assert_eq!(
        phase(&Board::from_fen("4k3/4p3/p1pp2pp/1p3p2/8/5P2/2PPP1PP/4K3 w - -").unwrap()).0,
        0
    );
}
