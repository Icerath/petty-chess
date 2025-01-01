use std::{
    fmt,
    ops::{Index, IndexMut},
    str::FromStr,
};

use crate::prelude::*;

bounded_int! { pub struct Square { 64 } }

impl Square {
    #[must_use]
    #[inline]
    pub const fn new(rank: Rank, file: File) -> Self {
        Self(file.u8() + rank.u8() * 8)
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
        unsafe { Square::from_int_unchecked(FLIPPED[self]) }
    }
    #[must_use]
    #[inline]
    pub const fn file(self) -> File {
        File(self.u8() % 8)
    }
    #[must_use]
    #[inline]
    pub const fn rank(self) -> Rank {
        Rank(self.u8() / 8)
    }
    #[must_use]
    #[inline]
    pub fn add_rank(self, rank: i8) -> Option<Self> {
        Some(Self::new(self.rank().add_int_signed(rank)?, self.file()))
    }
    #[must_use]
    #[inline]
    pub fn add_file(self, file: i8) -> Option<Self> {
        let file = self.file().add_int_signed(file)?;
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
        self.file().u8().abs_diff(other.file().u8()) + self.rank().u8().abs_diff(other.rank().u8())
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
    #[track_caller]
    pub fn passed_pawn_mask(self, side: Side) -> Bitboard {
        let (file, rank) = (self.file(), self.rank());
        let mut mask = file.add_int(1).map_or(Bitboard::EMPTY, File::mask)
            | file.sub_int(1).map_or(Bitboard::EMPTY, File::mask);
        match side {
            Side::White => mask.0 <<= (rank.u8() + 1) * 8,
            Side::Black => mask.0 >>= (8 - rank.u8()) * 8,
        }
        mask
    }
    #[inline]
    #[must_use]
    pub fn outpost_mask(self, side: Side) -> Bitboard {
        let (file, rank) = (self.file(), self.rank());
        let mut mask = file.add_int(1).map_or(Bitboard::EMPTY, File::mask)
            | file.sub_int(1).map_or(Bitboard::EMPTY, File::mask);
        match side {
            Side::White => mask.0 = mask.0.checked_shl((rank.u32() + 1) * 8).unwrap_or_default(),
            Side::Black => mask.0 = mask.0.checked_shr((8 - rank.u32()) * 8).unwrap_or_default(),
        }
        mask
    }
}

macro_rules! impl_file_rank {
    ($ty: ty) => {
        impl $ty {
            #[inline]
            #[must_use]
            pub fn distance_from_center(self) -> u8 {
                const OUTPUTS: [u8; 8] = [3, 2, 1, 0, 0, 1, 2, 3];
                OUTPUTS[self.usize()]
            }
            #[must_use]
            #[inline]
            pub fn relative_to(self, side: Side) -> Self {
                match side {
                    Side::White => self,
                    Side::Black => Self(7 - self.u8()),
                }
            }
        }
    };
    ($($ty: ty),+) => {
        $(impl_file_rank!($ty);)+
    };
}

bounded_int! { pub struct Rank { 8 } }
bounded_int! { pub struct File { 8 } }

impl_file_rank!(File, Rank);

macro_rules! define_file_consts {
    ($name: ident = $num: literal) => {
        pub const $name: Self = Self($num);
    };
    ($($name: ident $num: literal),+) => {
        $(define_file_consts!($name = $num);)+
    };
}

impl File {
    pub const ALL: [Self; 8] =
        [Self::A, Self::B, Self::C, Self::D, Self::E, Self::F, Self::G, Self::H];
    define_file_consts!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7);
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
        FILES[self.usize()]
    }

    const fn compute_mask(self) -> Bitboard {
        Bitboard(
            (1 << self.u8())
                + (1 << (8 + self.u8()))
                + (1 << (16 + self.u8()))
                + (1 << (24 + self.u8()))
                + (1 << (32 + self.u8()))
                + (1 << (40 + self.u8()))
                + (1 << (48 + self.u8()))
                + (1 << (56 + self.u8())),
        )
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
            .map(|index| unsafe { Self::from_int_unchecked(index as u8) })
            .ok_or(InvalidSquare)
    }
}

macro_rules! define_consts {
    ($name: ident = $num: literal) => {
        pub const $name: Self = Self($num);
    };
    ($($name: ident $num: literal),+,) => {
        $(define_consts!($name = $num);)+
    };
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

    define_consts!(
        A1  0, B1  1, C1  2, D1  3, E1  4, F1  5, G1  6, H1  7,
        A2  8, B2  9, C2 10, D2 11, E2 12, F2 13, G2 14, H2 15,
        A3 16, B3 17, C3 18, D3 19, E3 20, F3 21, G3 22, H3 23,
        A4 24, B4 25, C4 26, D4 27, E4 28, F4 29, G4 30, H4 31,
        A5 32, B5 33, C5 34, D5 35, E5 36, F5 37, G5 38, H5 39,
        A6 40, B6 41, C6 42, D6 43, E6 44, F6 45, G6 46, H6 47,
        A7 48, B7 49, C7 50, D7 51, E7 52, F7 53, G7 54, H7 55,
        A8 56, B8 57, C8 58, D8 59, E8 60, F8 61, G8 62, H8 63,
    );
}

#[test]
fn test_manhattan_distance() {
    assert_eq!(Square::A1.manhattan_distance(Square::H8), 14);
    assert_eq!(Square::E2.manhattan_distance(Square::E2), 0);
}

#[test]
fn test_square_flip() {
    for sq in Square::all() {
        assert_eq!(Square::new(Rank(7 - sq.rank().u8()), sq.file()), sq.flip());
    }
}
