use crate::prelude::*;

pub const ROOK: usize = 4096;
pub const BISHOP: usize = 512;

static ROOK_TABLES: [SquareTables<ROOK>; 64] =
    unsafe { std::mem::transmute(*include_bytes!("magic_rook_tables.bin")) };

static BISHOP_TABLES: [SquareTables<BISHOP>; 64] =
    unsafe { std::mem::transmute(*include_bytes!("magic_bishop_tables.bin")) };

#[must_use]
pub fn rook_attacks(sq: Square, occupancy: Bitboard) -> Bitboard {
    ROOK_TABLES[sq.usize()].get_attacks(occupancy)
}

#[must_use]
pub fn bishop_attacks(sq: Square, occupancy: Bitboard) -> Bitboard {
    BISHOP_TABLES[sq.usize()].get_attacks(occupancy)
}

#[must_use]
pub fn queen_attacks(sq: Square, occupancy: Bitboard) -> Bitboard {
    BISHOP_TABLES[sq.usize()].get_attacks(occupancy)
        | ROOK_TABLES[sq.usize()].get_attacks(occupancy)
}

#[repr(C)]
struct SquareTables<const PIECE: usize> {
    magic: u64,
    mask: u64,
    shift: u32,
    attacks: [u64; PIECE],
}

impl<const PIECE: usize> SquareTables<PIECE> {
    fn get_attacks(&self, mut occupancy: Bitboard) -> Bitboard {
        occupancy.0 &= self.mask;
        let index = (occupancy.0.wrapping_mul(self.magic) >> self.shift) as usize;
        Bitboard(unsafe { *self.attacks.get_unchecked(index) })
    }
}
