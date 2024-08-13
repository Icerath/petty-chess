use std::{
    fmt,
    ops::{Mul, Neg},
};

// A Eval in centipawns
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Eval(pub i32);

impl Eval {
    pub const INFINITY: Self = Self(i32::MAX);
    pub const MATE: Self = Self(i32::MAX - 1);

    const NEG_INF: Self = Self(-Self::INFINITY.0);
    const NEG_MATE: Self = Self(-Self::MATE.0);
}

impl Mul<f32> for Eval {
    type Output = Eval;
    fn mul(self, rhs: f32) -> Self::Output {
        Self((self.0 as f32 * rhs) as i32)
    }
}

impl Neg for Eval {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl fmt::Debug for Eval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut tuple = f.debug_tuple("Eval");
        match *self {
            Self::INFINITY => tuple.field(&"infinity"),
            Self::NEG_INF => tuple.field(&"-infinity"),
            Self::MATE => tuple.field(&"mate"),
            Self::NEG_MATE => tuple.field(&"-mate"),
            _ => tuple.field(&self.0),
        }
        .finish()
    }
}
