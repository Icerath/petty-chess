use crate::prelude::*;

pub const ROOK: usize = 4096;
pub const BISHOP: usize = 512;

static ROOK_TABLES: [SquareTables<ROOK>; 64] =
    unsafe { std::mem::transmute(*include_bytes!("magic_rook_tables.bin")) };

static BISHOP_TABLES: [SquareTables<BISHOP>; 64] =
    unsafe { std::mem::transmute(*include_bytes!("magic_bishop_tables.bin")) };

#[must_use]
pub fn rook_attacks(sq: Square, occupancy: Bitboard) -> Bitboard {
    ROOK_TABLES[sq].get_attacks(occupancy)
}

#[must_use]
pub fn bishop_attacks(sq: Square, occupancy: Bitboard) -> Bitboard {
    BISHOP_TABLES[sq].get_attacks(occupancy)
}

#[must_use]
pub fn queen_attacks(sq: Square, occupancy: Bitboard) -> Bitboard {
    Bitboard(rook_attacks(sq, occupancy).0 | bishop_attacks(sq, occupancy).0)
}

#[repr(C)]
struct SquareTables<const PIECE: usize> {
    magic: u64,
    mask: Bitboard,
    shift: u32,
    attacks: [Bitboard; PIECE],
}

impl<const PIECE: usize> SquareTables<PIECE> {
    const fn get_attacks(&self, mut occupancy: Bitboard) -> Bitboard {
        occupancy.0 &= self.mask.0;
        let index = (occupancy.0.wrapping_mul(self.magic) >> self.shift) as usize;
        unsafe { std::hint::assert_unchecked(index < PIECE) };
        self.attacks[index]
    }
}
