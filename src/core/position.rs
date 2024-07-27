use std::{fmt, str::FromStr};

#[derive(Default, Clone, Copy, PartialEq)]
pub struct Pos(pub i8);

impl Pos {
    #[must_use]
    #[inline]
    pub const fn new(rank: Rank, file: File) -> Self {
        Self(file.0 + rank.0 * 8)
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct File(pub i8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rank(pub i8);

impl Rank {
    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    #[inline]
    pub fn checked_add(self, rhs: i8) -> Option<Self> {
        let out = self.0 + rhs;
        (0..8).contains(&out).then_some(Self(out))
    }
}

impl File {
    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    #[inline]
    pub fn checked_add(self, rhs: i8) -> Option<Self> {
        let out = self.0 + rhs;
        (0..8).contains(&out).then_some(Self(out))
    }
}

impl fmt::Debug for Pos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Self::SQUARES[self.0 as usize])
    }
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

#[derive(Debug)]
pub struct InvalidPos;

impl fmt::Display for InvalidPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

impl std::error::Error for InvalidPos {}

impl FromStr for Pos {
    type Err = InvalidPos;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::SQUARES
            .iter()
            .position(|&square| square == input)
            .map(|index| Pos(index as i8))
            .ok_or(InvalidPos)
    }
}

impl Pos {
    #[must_use]
    pub fn algebraic(self) -> &'static str {
        Self::SQUARES[self.0 as usize]
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
