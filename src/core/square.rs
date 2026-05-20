use std::{fmt, str::FromStr};

use crate::prelude::*;

pod_enum! {
    pub enum Square {
        A1, B1, C1, D1, E1, F1, G1, H1,
        A2, B2, C2, D2, E2, F2, G2, H2,
        A3, B3, C3, D3, E3, F3, G3, H3,
        A4, B4, C4, D4, E4, F4, G4, H4,
        A5, B5, C5, D5, E5, F5, G5, H5,
        A6, B6, C6, D6, E6, F6, G6, H6,
        A7, B7, C7, D7, E7, F7, G7, H7,
        A8, B8, C8, D8, E8, F8, G8, H8,
    }
}
pod_enum! { pub enum Rank { _1, _2, _3, _4, _5, _6, _7, _8 } }
pod_enum! { pub enum File { A, B, C, D, E, F, G, H } }

impl Square {
    pub const ALL: [Self; 64] = {
        let mut all = [Self::A1; 64];
        let mut i = 0;
        while i < 64 {
            all[i as usize] = Self::from_int(i).unwrap();
            i += 1;
        }
        all
    };

    #[must_use]
    pub const fn new(rank: Rank, file: File) -> Self {
        unsafe { Self::from_int_unchecked(file as u8 + rank as u8 * 8) }
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
        unsafe { Self::from_int_unchecked(FLIPPED[self as usize]) }
    }

    #[must_use]
    pub const fn file(self) -> File {
        unsafe { File::from_int_unchecked(self as u8 % 8) }
    }

    #[must_use]
    pub const fn rank(self) -> Rank {
        unsafe { Rank::from_int_unchecked(self as u8 / 8) }
    }

    #[must_use]
    pub const fn add_rank(self, rank: i8) -> Option<Self> {
        let Some(_) = self.rank().add_int_signed(rank) else { return None };
        Some(unsafe { self.add_int_signed_unchecked(rank * 8) })
    }

    #[must_use]
    #[expect(clippy::missing_safety_doc, reason = "lazy")]
    pub const unsafe fn add_rank_unchecked(self, rank: i8) -> Self {
        unsafe { self.add_int_signed_unchecked(rank * 8) }
    }

    #[must_use]
    pub const fn add_file(self, file: i8) -> Option<Self> {
        let Some(_) = self.file().add_int_signed(file) else { return None };
        Some(unsafe { self.add_int_signed_unchecked(file) })
    }

    #[must_use]
    pub const fn manhattan_distance(self, other: Self) -> u8 {
        self.file().diff(other.file()) + self.rank().diff(other.rank())
    }

    #[must_use]
    pub const fn square_distance(self, other: Self) -> u8 {
        let file_diff = self.file().diff(other.file());
        let rank_diff = self.rank().diff(other.rank());
        if file_diff > rank_diff { file_diff } else { rank_diff }
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
        ][self as usize]
    }

    #[must_use]
    pub const fn passed_pawn_mask(self, side: Side) -> Bitboard {
        const LUT: [[Bitboard; 64]; 2] = {
            let mut black_lut = [Bitboard::EMPTY; 64];
            let mut i = 0;
            while i < 64 {
                black_lut[i as usize] =
                    Square::from_int(i).unwrap().compute_passed_pawn_mask(Black);
                i += 1;
            }

            let mut white_lut = [Bitboard::EMPTY; 64];
            let mut i = 0;
            while i < 64 {
                white_lut[i as usize] =
                    Square::from_int(i).unwrap().compute_passed_pawn_mask(White);
                i += 1;
            }
            [black_lut, white_lut]
        };
        LUT[side as usize][self as usize]
    }

    #[must_use]
    const fn compute_passed_pawn_mask(self, side: Side) -> Bitboard {
        let (file, rank) = (self.file(), self.rank());
        let mut mask = Bitboard(file.adjacency_mask().0 | file.mask().0);
        match side {
            Side::White => mask.0 = mask.0.wrapping_shl(((rank as u8 + 1) * 8) as u32),
            Side::Black => mask.0 = mask.0.wrapping_shr(((8 - rank as u8) * 8) as u32),
        }
        mask
    }

    #[must_use]
    pub const fn outpost_mask(self, side: Side) -> Bitboard {
        let (file, rank) = (self.file(), self.rank());
        let mask = file.adjacency_mask();
        let mask = match side {
            Side::White => mask.0.checked_shl((rank as u32 + 1) * 8),
            Side::Black => mask.0.checked_shr((8 - rank as u32) * 8),
        };
        match mask {
            Some(mask) => Bitboard(mask),
            None => Bitboard::EMPTY,
        }
    }

    #[must_use]
    pub(crate) const fn nearby3(self) -> Bitboard {
        const LUT: [Bitboard; 64] = {
            let mut lut = [Bitboard::EMPTY; 64];
            let mut sq = 0;
            while sq < 64 {
                lut[sq as usize] = Square::from_int(sq).unwrap().compute_nearby(3);
                sq += 1;
            }
            lut
        };
        LUT[self as usize]
    }

    #[must_use]
    pub(crate) const fn nearby2(self) -> Bitboard {
        const LUT: [Bitboard; 64] = {
            let mut lut = [Bitboard::EMPTY; 64];
            let mut sq = 0;
            while sq < 64 {
                lut[sq as usize] = Square::from_int(sq).unwrap().compute_nearby(2);
                sq += 1;
            }
            lut
        };
        LUT[self as usize]
    }

    const fn compute_nearby(self, distance: u8) -> Bitboard {
        let mut bitboard = Bitboard::EMPTY;
        let mut i = 0;
        while i < 64 {
            let sq = Self::from_int(i).unwrap();
            if sq as u8 != self as u8 && sq.square_distance(self) <= distance {
                bitboard.insert(sq);
            }
            i += 1;
        }
        bitboard
    }

    #[must_use]
    pub const fn mask(self) -> Bitboard {
        Bitboard(1 << self as u8)
    }
}

macro_rules! impl_file_rank {
    ($ty: ty) => {
        impl $ty {
            #[must_use]
            pub const fn distance_from_center(self) -> u8 {
                [3, 2, 1, 0, 0, 1, 2, 3][self as usize]
            }
            #[must_use]
            pub const fn relative_to(self, side: Side) -> Self {
                match side {
                    Side::White => self,
                    Side::Black => self.invert(),
                }
            }
            #[must_use]
            pub const fn diff(self, other: Self) -> u8 {
                (self as u8).abs_diff(other as u8)
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
        pub const $name: Self = unsafe { Self::from_int_unchecked($num) };
    };
    ($($name: ident $num: literal),+) => {
        $(define_file_consts!($name = $num);)+
    };
}

impl File {
    pub const ALL: [Self; 8] =
        [Self::A, Self::B, Self::C, Self::D, Self::E, Self::F, Self::G, Self::H];

    define_file_consts!(A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7);

    #[must_use]
    pub const fn mask(self) -> Bitboard {
        Bitboard(0x0101_0101_0101_0101 << self as u8)
    }

    #[must_use]
    pub const fn adjacency_mask(self) -> Bitboard {
        const LUT: [Bitboard; 8] =
            konst::array::from_fn!(|i| File::from_int(i as _).unwrap().compute_adjacency_mask());
        LUT[self as usize]
    }

    const fn compute_adjacency_mask(self) -> Bitboard {
        Bitboard(self.mask().shift_left().0 | self.mask().shift_right().0)
    }
}

impl Rank {
    pub const ALL: [Self; 8] = unsafe { std::mem::transmute([0u8, 1, 2, 3, 4, 5, 6, 7]) };

    #[must_use]
    pub const fn mask(self) -> Bitboard {
        Bitboard(0x0000_0000_0000_00FF << (self as u8 * 8))
    }
}

impl fmt::Debug for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Self::SQUARES[*self as usize])
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

    #[must_use]
    pub const fn algebraic(self) -> &'static str {
        Self::SQUARES[self as usize]
    }

    #[must_use]
    pub(crate) fn from_bstr(bstr: [u8; 2]) -> Option<Self> {
        let square_int =
            bstr[0].checked_sub(b'a')?.checked_add((bstr[1].checked_sub(b'1')?).checked_mul(8)?)?;
        Self::from_int(square_int)
    }

    #[must_use]
    pub const fn invert_rank(self) -> Self {
        Self::new(self.rank().invert(), self.file())
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
        assert_eq!(Square::new(sq.rank().invert(), sq.file()), sq.flip());
    }
}

#[test]
fn test_square_from_str() {
    for sq in Square::ALL {
        let sq_str = sq.algebraic();
        assert_eq!(sq_str.parse::<Square>().expect(sq_str), sq);
    }
}

#[test]
fn test_rank_mask() {
    let mask = Rank::from_int(1).unwrap().mask();
    let board = Board::start_pos();
    assert_eq!(board[Pawn] & mask, mask);

    for file in File::ALL {
        for rank in Rank::ALL {
            assert_eq!((file.mask() & rank.mask()).count(), 1);
            assert_eq!(file.mask().count(), 8);
            assert_eq!(rank.mask().count(), 8);
        }
    }
}

#[test]
fn test_passed_pawn_mask() {
    let expected = Bitboard::from_bstr(
        b"...XXX..\
          ...XXX..\
          ...XXX..\
          ...XXX..\
          ...XXX..\
          ...XXX..\
          ........\
          ........",
    );
    assert_eq!(Square::E2.passed_pawn_mask(White), expected);
}
