use std::{
    fmt,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not},
};

use crate::prelude::*;

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bitboard(pub u64);

impl Bitboard {
    pub const EMPTY: Self = Self(0);
    pub const ALL: Self = Self(u64::MAX);
    #[inline]
    pub fn insert(&mut self, sq: Square) {
        self.0 |= 1 << sq.int();
    }
    #[inline]
    pub fn remove(&mut self, sq: Square) {
        self.0 &= !(1 << sq.int());
    }
    #[inline]
    #[must_use]
    pub fn contains(self, sq: Square) -> bool {
        self.0 & (1 << sq.int()) > 0
    }
    #[inline]
    #[must_use]
    pub fn bitscan(self) -> Option<Square> {
        if self.is_empty() {
            return None;
        }
        unsafe { Some(self.bitscan_unchecked()) }
    }
    pub unsafe fn bitscan_unchecked(self) -> Square {
        unsafe { Square::new_int_unchecked(self.0.trailing_zeros() as u8) }
    }
    #[inline]
    pub fn bitscan_pop(&mut self) -> Option<Square> {
        let sq = self.bitscan()?;
        self.0 &= self.0 - 1;
        Some(sq)
    }
    #[inline]
    pub unsafe fn bitscan_pop_unchecked(&mut self) -> Square {
        let sq = unsafe { self.bitscan_unchecked() };
        self.0 &= self.0 - 1;
        sq
    }
    #[inline]
    #[must_use]
    pub fn rbitscan(self) -> Option<Square> {
        if self.is_empty() {
            return None;
        }
        Some(unsafe { self.rbitscan_unchecked() })
    }
    #[inline]
    #[must_use]
    pub unsafe fn rbitscan_unchecked(self) -> Square {
        unsafe { Square::new_int_unchecked(self.0.leading_zeros() as u8) }
    }
    #[inline]
    pub fn for_each<F: FnMut(Square)>(mut self, mut f: F) {
        while !self.is_empty() {
            f(unsafe { self.bitscan_pop_unchecked() });
        }
    }
    #[inline]
    #[must_use]
    pub fn count(self) -> u32 {
        self.0.count_ones()
    }
    #[inline]
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
    #[inline]
    #[must_use]
    pub fn contains_in_file(self, file: File) -> bool {
        (self & file.mask()).0 > 0
    }
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
