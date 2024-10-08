use std::{
    fmt,
    ops::{Add, Index, IndexMut, Sub},
    str::FromStr,
};

use crate::prelude::*;

#[derive(Default, Clone, Copy, PartialEq)]
pub struct Square(u8);

impl Square {
    /// # Safety
    /// int must be < 64
    #[must_use]
    #[inline]
    pub const unsafe fn new_int_unchecked(int: u8) -> Self {
        Self(int)
    }
    #[must_use]
    #[inline]
    pub const fn int(self) -> u8 {
        unsafe { std::hint::assert_unchecked(self.0 < 64) };
        self.0
    }
    #[must_use]
    #[inline]
    pub const fn new(rank: Rank, file: File) -> Self {
        assert!(file.0 < 8 && rank.0 < 8);
        Self(file.0 + rank.0 * 8)
    }
    #[must_use]
    #[inline]
    pub fn flip(self) -> Self {
        #[rustfmt::skip]
        const FLIPPED: [u8; 64] = [
            56, 57, 58, 59, 60, 61, 62, 63,
            48, 49, 50, 51, 52, 53, 54, 55,
            40, 41, 42, 43, 44, 45, 46, 47, 
            32, 33, 34, 35, 36, 37, 38, 39,
            24, 25, 26, 27, 28, 29, 30, 31, 
            16, 17, 18, 19, 20, 21, 22, 23,
             8,  9, 10, 11, 12, 13, 14, 15,
             0,  1,  2,  3,  4,  5,  6,  7,
        ];
        unsafe { Square::new_int_unchecked(FLIPPED[self]) }
    }
    #[must_use]
    #[inline]
    pub const fn file(self) -> File {
        File(self.0 % 8)
    }
    #[must_use]
    #[inline]
    pub const fn rank(self) -> Rank {
        Rank(self.0 / 8)
    }
    #[must_use]
    #[inline]
    pub fn add_rank(self, rank: i8) -> Option<Self> {
        let rank = self.rank().checked_add(rank)?;
        Some(Self::new(rank, self.file()))
    }
    #[must_use]
    #[inline]
    pub fn add_file(self, file: i8) -> Option<Self> {
        let file = self.file().checked_add(file)?;
        Some(Self::new(self.rank(), file))
    }
    #[inline]
    #[must_use]
    pub fn all() -> impl ExactSizeIterator<Item = Self> {
        (0..64).map(Self)
    }
    #[must_use]
    #[inline]
    pub fn manhattan_distance(self, other: Self) -> u8 {
        self.file().0.abs_diff(other.file().0) + self.rank().0.abs_diff(other.rank().0)
    }
    #[must_use]
    #[inline]
    pub fn centre_manhattan_distance(self) -> u8 {
        [
            3, 3, 3, 3, 3, 3, 3, 3, //
            3, 2, 2, 2, 2, 2, 2, 3, //
            3, 2, 1, 1, 1, 1, 2, 3, //
            3, 2, 1, 0, 0, 1, 2, 3, //
            3, 2, 1, 0, 0, 1, 2, 3, //
            3, 2, 1, 1, 1, 1, 2, 3, //
            3, 2, 2, 2, 2, 2, 2, 3, //
            3, 3, 3, 3, 3, 3, 3, 3, //
        ][self]
    }
    #[inline]
    #[must_use]
    /// # Panics
    /// Panics on debug builds when file is 0 or 7.
    /// Instead produces incorrect masks on release
    pub fn passed_pawn_mask(self, side: Side) -> Bitboard {
        let (file, rank) = (self.file(), self.rank());
        let mut mask = file.mask() | (file + 1).mask() | (file - 1).mask();
        match side {
            Side::White => mask.0 <<= (rank.0 + 1) * 8,
            Side::Black => mask.0 >>= (8 - rank.0) * 8,
        }
        mask
    }
    /// # Panics
    /// Panics on debug builds when file is 0 or 7.
    /// Instead produces incorrect masks on release
    #[inline]
    #[must_use]
    pub fn outpost_mask(self, side: Side) -> Bitboard {
        let (file, rank) = (self.file(), self.rank());
        let mut mask = (file + 1).mask() | (file - 1).mask();
        match side {
            Side::White => mask.0 = mask.0.checked_shl((rank.0 as u32 + 1) * 8).unwrap_or_default(),
            Side::Black => mask.0 = mask.0.checked_shr((8 - rank.0 as u32) * 8).unwrap_or_default(),
        }
        mask
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct File(pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rank(pub u8);

impl Rank {
    #[must_use]
    #[inline]
    pub fn checked_add(self, rhs: i8) -> Option<Self> {
        let out = self.0 as i8 + rhs;
        (0..8).contains(&out).then_some(Self(out as u8))
    }
    #[must_use]
    #[inline]
    pub fn relative_to(self, side: Side) -> Self {
        match side {
            Side::White => self,
            Side::Black => Self(7 - self.0),
        }
    }
}

impl<T> Index<Square> for [T] {
    type Output = T;
    #[inline]
    fn index(&self, sq: Square) -> &Self::Output {
        &self[usize::from(sq)]
    }
}

impl<T> IndexMut<Square> for [T] {
    #[inline]
    fn index_mut(&mut self, sq: Square) -> &mut Self::Output {
        &mut self[usize::from(sq)]
    }
}

impl File {
    #[must_use]
    #[inline]
    pub fn checked_add(self, rhs: i8) -> Option<Self> {
        let out = self.0 as i8 + rhs;
        (0..8).contains(&out).then_some(Self(out as u8))
    }
    #[must_use]
    #[inline]
    // Produces a mask representing a file from 0..8
    // Produces an empty bitboard for File(-1) and File(8)
    // Oher file values are undefined behaviour
    pub fn mask(self) -> Bitboard {
        const FILES: [Bitboard; 8] = [
            File(0).compute_mask(),
            File(1).compute_mask(),
            File(2).compute_mask(),
            File(3).compute_mask(),
            File(4).compute_mask(),
            File(5).compute_mask(),
            File(6).compute_mask(),
            File(7).compute_mask(),
        ];
        FILES.get(usize::from(self.0)).copied().unwrap_or(Bitboard::EMPTY)
    }

    const fn compute_mask(self) -> Bitboard {
        Bitboard(
            (1 << self.0)
                + (1 << (8 + self.0))
                + (1 << (16 + self.0))
                + (1 << (24 + self.0))
                + (1 << (32 + self.0))
                + (1 << (40 + self.0))
                + (1 << (48 + self.0))
                + (1 << (56 + self.0)),
        )
    }
}

impl Add<i8> for File {
    type Output = Self;
    fn add(self, rhs: i8) -> Self::Output {
        Self(self.0.wrapping_add_signed(rhs))
    }
}

impl Sub<i8> for File {
    type Output = Self;
    fn sub(self, rhs: i8) -> Self::Output {
        Self(self.0.wrapping_add_signed(-rhs))
    }
}

impl fmt::Debug for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Self::SQUARES[*self])
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

#[derive(Debug)]
pub struct InvalidSquare;

impl fmt::Display for InvalidSquare {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

impl std::error::Error for InvalidSquare {}

impl FromStr for Square {
    type Err = InvalidSquare;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::SQUARES
            .iter()
            .position(|&sq| sq == input)
            .map(|index| unsafe { Self::new_int_unchecked(index as u8) })
            .ok_or(InvalidSquare)
    }
}

impl Square {
    #[must_use]
    pub fn algebraic(self) -> &'static str {
        Self::SQUARES[self]
    }
    #[rustfmt::skip]
    pub const SQUARES: [&'static str; 64] = [
        "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1",
        "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
        "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3",
        "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
        "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5",
        "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
        "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7",
        "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8",
    ];

    pub const A1: Self = Self(0);
    pub const B1: Self = Self(1);
    pub const C1: Self = Self(2);
    pub const D1: Self = Self(3);
    pub const E1: Self = Self(4);
    pub const F1: Self = Self(5);
    pub const G1: Self = Self(6);
    pub const H1: Self = Self(7);

    pub const A2: Self = Self(8);
    pub const B2: Self = Self(9);
    pub const C2: Self = Self(10);
    pub const D2: Self = Self(11);
    pub const E2: Self = Self(12);
    pub const F2: Self = Self(13);
    pub const G2: Self = Self(14);
    pub const H2: Self = Self(15);

    pub const A3: Self = Self(16);
    pub const B3: Self = Self(17);
    pub const C3: Self = Self(18);
    pub const D3: Self = Self(19);
    pub const E3: Self = Self(20);
    pub const F3: Self = Self(21);
    pub const G3: Self = Self(22);
    pub const H3: Self = Self(23);

    pub const A4: Self = Self(24);
    pub const B4: Self = Self(25);
    pub const C4: Self = Self(26);
    pub const D4: Self = Self(27);
    pub const E4: Self = Self(28);
    pub const F4: Self = Self(29);
    pub const G4: Self = Self(30);
    pub const H4: Self = Self(31);

    pub const A5: Self = Self(32);
    pub const B5: Self = Self(33);
    pub const C5: Self = Self(34);
    pub const D5: Self = Self(35);
    pub const E5: Self = Self(36);
    pub const F5: Self = Self(37);
    pub const G5: Self = Self(38);
    pub const H5: Self = Self(39);

    pub const A6: Self = Self(40);
    pub const B6: Self = Self(41);
    pub const C6: Self = Self(42);
    pub const D6: Self = Self(43);
    pub const E6: Self = Self(44);
    pub const F6: Self = Self(45);
    pub const G6: Self = Self(46);
    pub const H6: Self = Self(47);

    pub const A7: Self = Self(48);
    pub const B7: Self = Self(49);
    pub const C7: Self = Self(50);
    pub const D7: Self = Self(51);
    pub const E7: Self = Self(52);
    pub const F7: Self = Self(53);
    pub const G7: Self = Self(54);
    pub const H7: Self = Self(55);

    pub const A8: Self = Self(56);
    pub const B8: Self = Self(57);
    pub const C8: Self = Self(58);
    pub const D8: Self = Self(59);
    pub const E8: Self = Self(60);
    pub const F8: Self = Self(61);
    pub const G8: Self = Self(62);
    pub const H8: Self = Self(63);
}

macro_rules! impl_try_from {
    ($($int:ident),*) => {
        $( impl_try_from!(@single $int);)*
    };

    (@single $int: ident) => {
        impl TryFrom<$int> for Square {
            type Error = $int;
            #[allow(clippy::cast_possible_truncation)]
            #[allow(clippy::cast_sign_loss)]
            #[inline]
            fn try_from(value: $int) -> Result<Self, Self::Error> {
                match value {
                    0..81 => Ok(unsafe { Self::new_int_unchecked(value as u8) }),
                    _ => Err(value),
                }
            }
        }
    };
}

macro_rules! impl_into {
    ($($int:ident),*) => {
        $( impl_into!(@single $int);)*
    };
    (@single $int: ident) => {
        impl From<Square> for $int {
            #[allow(clippy::cast_possible_wrap)]
            #[must_use]
            #[inline]
            fn from(square: Square) -> $int {
                square.int() as $int
            }
        }
    };
}

impl_try_from!(u8, i8, u16, i16, u32, i32, usize);
impl_into!(u8, i8, u16, i16, u32, i32, usize);

#[test]
fn test_manhattan_distance() {
    assert_eq!(Square::A1.manhattan_distance(Square::H8), 14);
    assert_eq!(Square::E2.manhattan_distance(Square::E2), 0);
}


#[test]
fn test_square_flip() {
    for sq in Square::all() {
        assert_eq!(Square::new(Rank(7 - sq.rank().0), sq.file()), sq.flip());
    }
}