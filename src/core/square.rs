use std::{fmt, str::FromStr};

use crate::prelude::*;

bounded_int! { pub struct Square { 64 } }
bounded_int! { pub struct Rank { 8 } }
bounded_int! { pub struct File { 8 } }

impl Square {
    pub const ALL: [Self; 64] = {
        let mut all = [Self::A1; 64];
        let mut i = 0;
        while i < 64 {
            all[i as usize] = Square::from_int(i).unwrap();
            i += 1;
        }
        all
    };

    #[must_use]
    pub const fn new(rank: Rank, file: File) -> Self {
        Self(file.u8() + rank.u8() * 8)
    }

    #[must_use]
    pub const fn flip(self) -> Self {
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
        unsafe { Self::from_int_unchecked(FLIPPED[self.usize()]) }
    }

    #[must_use]
    pub const fn file(self) -> File {
        File(self.u8() % 8)
    }

    #[must_use]
    pub const fn rank(self) -> Rank {
        Rank(self.u8() / 8)
    }

    #[must_use]
    pub const fn add_rank(self, rank: i8) -> Option<Self> {
        let Some(rank) = self.rank().add_int_signed(rank) else { return None };
        Some(Self::new(rank, self.file()))
    }

    #[must_use]
    pub const fn add_file(self, file: i8) -> Option<Self> {
        let Some(file) = self.file().add_int_signed(file) else { return None };
        Some(Self::new(self.rank(), file))
    }

    #[must_use]
    pub const fn manhattan_distance(self, other: Self) -> u8 {
        self.file().u8().abs_diff(other.file().u8()) + self.rank().u8().abs_diff(other.rank().u8())
    }

    #[must_use]
    pub const fn centre_manhattan_distance(self) -> u8 {
        [
            3, 3, 3, 3, 3, 3, 3, 3, //
            3, 2, 2, 2, 2, 2, 2, 3, //
            3, 2, 1, 1, 1, 1, 2, 3, //
            3, 2, 1, 0, 0, 1, 2, 3, //
            3, 2, 1, 0, 0, 1, 2, 3, //
            3, 2, 1, 1, 1, 1, 2, 3, //
            3, 2, 2, 2, 2, 2, 2, 3, //
            3, 3, 3, 3, 3, 3, 3, 3, //
        ][self.usize()]
    }

    #[must_use]
    pub const fn passed_pawn_mask(self, side: Side) -> Bitboard {
        let (file, rank) = (self.file(), self.rank());
        let mut mask = file.adjacency_mask();
        match side {
            Side::White => mask.0 <<= (rank.u8() + 1) * 8,
            Side::Black => mask.0 >>= (8 - rank.u8()) * 8,
        }
        mask
    }

    #[must_use]
    pub const fn outpost_mask(self, side: Side) -> Bitboard {
        let (file, rank) = (self.file(), self.rank());
        let mask = file.adjacency_mask();
        let mask = match side {
            Side::White => mask.0.checked_shl((rank.u32() + 1) * 8),
            Side::Black => mask.0.checked_shr((8 - rank.u32()) * 8),
        };
        match mask {
            Some(mask) => Bitboard(mask),
            None => Bitboard::EMPTY,
        }
    }
}

macro_rules! impl_file_rank {
    ($ty: ty) => {
        impl $ty {
            #[must_use]
            pub const fn distance_from_center(self) -> u8 {
                [3, 2, 1, 0, 0, 1, 2, 3][self.usize()]
            }
            #[must_use]
            pub const fn relative_to(self, side: Side) -> Self {
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

impl File {
    #[must_use]
    pub const fn adjacency_mask(self) -> Bitboard {
        unsafe {
            match self {
                Self::A => self.add_int_unchecked(1).mask(),
                Self::H => self.sub_int_unchecked(1).mask(),
                _ => Bitboard(
                    self.sub_int_unchecked(1).mask().0 | self.add_int_unchecked(1).mask().0,
                ),
            }
        }
    }

    #[must_use]
    // Produces a mask representing a file from 0..8
    // Produces an empty bitboard for File(-1) and File(8)
    // Oher file values are undefined behaviour
    pub const fn mask(self) -> Bitboard {
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
        write!(f, "{}", Self::SQUARES[self.usize()])
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
        let bstr = input.as_bytes().try_into().map_err(|_| InvalidSquare)?;
        Self::from_bstr(bstr).ok_or(InvalidSquare)
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

    #[must_use]
    pub const fn algebraic(self) -> &'static str {
        Self::SQUARES[self.usize()]
    }

    #[must_use]
    pub(crate) fn from_bstr(bstr: [u8; 2]) -> Option<Self> {
        let square_int =
            bstr[0].checked_sub(b'a')?.checked_add((bstr[1].checked_sub(b'1')?).checked_mul(8)?)?;
        Square::from_int(square_int)
    }
}

#[test]
fn test_manhattan_distance() {
    assert_eq!(Square::A1.manhattan_distance(Square::H8), 14);
    assert_eq!(Square::E2.manhattan_distance(Square::E2), 0);
}

#[test]
fn test_square_flip() {
    for sq in Square::ALL {
        assert_eq!(Square::new(Rank(7 - sq.rank().u8()), sq.file()), sq.flip());
    }
}

#[test]
fn test_square_from_str() {
    for sq in Square::ALL {
        let sq_str = sq.algebraic();
        assert_eq!(sq_str.parse::<Square>().expect(sq_str), sq);
    }
}
