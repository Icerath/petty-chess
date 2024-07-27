use std::fmt;

use crate::prelude::*;

#[derive(Clone, Copy)]
pub struct Bitboard(pub u64);

impl Bitboard {
    #[inline]
    pub fn insert(&mut self, pos: Pos) {
        self.0 |= 1 << pos.0;
    }
    #[inline]
    pub fn remove(&mut self, pos: Pos) {
        self.0 &= !(1 << pos.0);
    }
    #[inline]
    #[must_use]
    pub fn contains(&self, pos: Pos) -> bool {
        self.0 & (1 << pos.0) > 0
    }
}

impl FromIterator<Pos> for Bitboard {
    fn from_iter<T: IntoIterator<Item = Pos>>(iter: T) -> Self {
        let mut ret = Self(0);
        ret.extend(iter);
        ret
    }
}

impl Extend<Pos> for Bitboard {
    fn extend<T: IntoIterator<Item = Pos>>(&mut self, iter: T) {
        iter.into_iter().for_each(|pos| self.insert(pos));
    }
}

impl fmt::Debug for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

impl fmt::Display for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rank in (0..8).rev() {
            for file in 0..8 {
                let pos = Pos::new(Rank(rank), File(file));
                write!(f, "{}", self.contains(pos) as u8)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
