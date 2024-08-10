use std::{
    fmt,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not},
};

use crate::prelude::*;

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bitboard(pub u64);

impl Bitboard {
    #[inline]
    pub fn insert(&mut self, sq: Square) {
        self.0 |= 1 << sq.0;
    }
    #[inline]
    pub fn remove(&mut self, sq: Square) {
        self.0 &= !(1 << sq.0);
    }
    #[inline]
    #[must_use]
    pub fn contains(self, sq: Square) -> bool {
        self.0 & (1 << sq.0) > 0
    }
    #[inline]
    #[must_use]
    pub fn bitscan(self) -> Square {
        Square(self.0.trailing_zeros() as i8)
    }
    #[inline]
    pub fn for_each<F: FnMut(Square)>(mut self, mut f: F) {
        while self.0 > 0 {
            let next = self.bitscan();
            f(next);
            self.remove(next);
        }
    }
    #[inline]
    #[must_use]
    pub fn count(self) -> u32 {
        self.0.count_ones()
    }

    #[inline]
    #[must_use]
    pub fn contains_in_file(&self, file: File) -> bool {
        self.filter_file(file).0 > 0
    }
    #[inline]
    #[must_use]
    pub fn filter_file(self, file: File) -> Bitboard {
        Self::FILES[file.0 as usize] & self
    }
    const FILES: [Self; 8] = [
        file_bitboard(File(0)),
        file_bitboard(File(1)),
        file_bitboard(File(2)),
        file_bitboard(File(3)),
        file_bitboard(File(4)),
        file_bitboard(File(5)),
        file_bitboard(File(6)),
        file_bitboard(File(7)),
    ];
}
const fn file_bitboard(file: File) -> Bitboard {
    Bitboard(
        (1 << file.0)
            + (1 << (8 + file.0))
            + (1 << (16 + file.0))
            + (1 << (24 + file.0))
            + (1 << (32 + file.0))
            + (1 << (40 + file.0))
            + (1 << (48 + file.0))
            + (1 << (56 + file.0)),
    )
}

impl Not for Bitboard {
    type Output = Self;
    #[inline]
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl BitAnd for Bitboard {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for Bitboard {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

impl BitOr for Bitboard {
    type Output = Self;
    #[inline]
    #[must_use]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Bitboard {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

impl FromIterator<Square> for Bitboard {
    #[inline]
    fn from_iter<T: IntoIterator<Item = Square>>(iter: T) -> Self {
        let mut ret = Self(0);
        ret.extend(iter);
        ret
    }
}

impl Extend<Square> for Bitboard {
    #[inline]
    fn extend<T: IntoIterator<Item = Square>>(&mut self, iter: T) {
        iter.into_iter().for_each(|sq| self.insert(sq));
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
                let sq = Square::new(Rank(rank), File(file));
                write!(f, "{}", self.contains(sq) as u8)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
