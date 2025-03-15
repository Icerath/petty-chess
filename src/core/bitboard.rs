use std::{
    fmt,
    iter::FusedIterator,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not},
};

use crate::prelude::*;

#[repr(transparent)]
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bitboard(pub u64);

impl Bitboard {
    pub const ALL: Self = Self(u64::MAX);
    pub const EMPTY: Self = Self(0);

    #[cfg(test)]
    #[must_use]
    pub(crate) const fn from_bstr(bstr: &[u8; 64]) -> Self {
        let mut sq = 0;
        let mut bb = Self::EMPTY;
        while sq < 64 {
            if bstr[sq as usize] == b'X' {
                bb.insert(Square::from_int(sq).unwrap().invert_rank());
            }
            sq += 1;
        }
        bb
    }

    pub const fn insert(&mut self, sq: Square) {
        self.0 |= sq.mask().0;
    }

    pub const fn remove(&mut self, sq: Square) {
        self.0 &= !sq.mask().0;
    }

    #[must_use]
    pub const fn contains(self, sq: Square) -> bool {
        self.0 & sq.mask().0 > 0
    }

    #[must_use]
    pub const fn bitscan(self) -> Option<Square> {
        if self.is_empty() {
            return None;
        }
        unsafe { Some(self.bitscan_unchecked()) }
    }

    #[must_use]
    /// # Safety
    /// bitboard must not be empty
    pub const unsafe fn bitscan_unchecked(self) -> Square {
        unsafe { std::hint::assert_unchecked(self.0 != 0) };
        unsafe { Square::from_int_unchecked(self.0.trailing_zeros() as u8) }
    }

    pub const fn bitscan_pop(&mut self) -> Option<Square> {
        let Some(sq) = self.bitscan() else { return None };
        self.0 &= self.0 - 1;
        Some(sq)
    }

    /// # Safety
    /// bitboard must not be empty
    pub const unsafe fn bitscan_pop_unchecked(&mut self) -> Square {
        let sq = unsafe { self.bitscan_unchecked() };
        self.0 &= self.0 - 1;
        sq
    }

    #[must_use]
    pub const fn rbitscan(self) -> Option<Square> {
        if self.is_empty() {
            return None;
        }
        Some(unsafe { self.rbitscan_unchecked() })
    }

    #[must_use]
    /// # Safety
    /// bitboard must not be empty
    pub const unsafe fn rbitscan_unchecked(self) -> Square {
        unsafe { Square::from_int_unchecked(self.0.leading_zeros() as u8) }
    }

    pub fn for_each<F: FnMut(Square)>(mut self, mut f: F) {
        while !self.is_empty() {
            f(unsafe { self.bitscan_pop_unchecked() });
        }
    }

    #[must_use]
    pub const fn count(self) -> u8 {
        self.0.count_ones() as u8
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    #[must_use]
    pub const fn from_ref(int: &u64) -> &Self {
        unsafe { &*std::ptr::from_ref(int).cast::<Self>() }
    }

    #[must_use]
    pub const fn from_mut(int: &mut u64) -> &mut Self {
        unsafe { &mut *std::ptr::from_mut(int).cast::<Self>() }
    }
}

impl Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitXor<Square> for Bitboard {
    type Output = Self;

    fn bitxor(self, rhs: Square) -> Self::Output {
        Self(self.0 ^ rhs.mask().0)
    }
}

impl BitXorAssign<Square> for Bitboard {
    fn bitxor_assign(&mut self, rhs: Square) {
        self.0 ^= rhs.mask().0;
    }
}

impl FromIterator<Square> for Bitboard {
    fn from_iter<T: IntoIterator<Item = Square>>(iter: T) -> Self {
        let mut ret = Self(0);
        ret.extend(iter);
        ret
    }
}

impl Extend<Square> for Bitboard {
    fn extend<T: IntoIterator<Item = Square>>(&mut self, iter: T) {
        iter.into_iter().for_each(|sq| self.insert(sq));
    }
}

impl fmt::Debug for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rank in (0..8).rev() {
            for file in 0..8 {
                let sq = Square::new(Rank::from_int(rank).unwrap(), File::from_int(file).unwrap());
                write!(f, "{}", self.contains(sq) as u8)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl IntoIterator for Bitboard {
    type IntoIter = IntoIter;
    type Item = Square;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self)
    }
}

#[repr(transparent)]
pub struct IntoIter(Bitboard);

impl Iterator for IntoIter {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.bitscan_pop()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }

    fn count(self) -> usize {
        self.len()
    }

    fn for_each<F>(self, f: F)
    where
        F: FnMut(Self::Item),
    {
        self.0.for_each(f);
    }

    fn fold<B, F>(mut self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        let mut accum = init;
        while !self.0.is_empty() {
            accum = f(accum, unsafe { self.0.bitscan_pop_unchecked() });
        }
        accum
    }
}

impl ExactSizeIterator for IntoIter {
    fn len(&self) -> usize {
        self.0.count() as usize
    }
}

impl FusedIterator for IntoIter {}

#[test]
fn test_iter() {
    let bitboard: Bitboard = [Square::A1, Square::A2].into_iter().collect();
    let mut iter = bitboard.into_iter();
    assert_eq!(iter.next(), Some(Square::A1));
    assert_eq!(iter.next(), Some(Square::A2));
    assert_eq!(iter.next(), None);
}

#[test]
fn test_from_bstr() {
    assert_eq!(Bitboard::from_bstr(&[0; 64]), Bitboard::EMPTY);
    assert_eq!(Bitboard::from_bstr(&[b'X'; 64]), Bitboard::ALL);
}
