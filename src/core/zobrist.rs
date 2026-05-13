use std::{fmt, hint::assert_unchecked};

use crate::prelude::*;

#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct Zobrist(u64);

impl Zobrist {
    pub const DEFAULT: Self = Self(0);

    pub fn xor_side_to_move(&mut self) {
        self.0 ^= KEYS.side;
    }

    pub fn xor_piece(&mut self, sq: Square, piece: Piece) {
        self.0 ^= KEYS.piece[piece as usize][sq as usize];
    }

    pub fn xor_can_castle(&mut self, can_castle: CanCastle) {
        unsafe { assert_unchecked((can_castle.bits() as usize) < KEYS.castle.len()) };
        self.0 ^= KEYS.castle[can_castle.bits() as usize];
    }

    pub fn xor_en_passant(&mut self, sq: Square) {
        self.0 ^= KEYS.en_passant[sq as usize];
    }
}

impl fmt::Debug for Zobrist {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

struct Rng(u64);

impl Rng {
    #[expect(clippy::cast_possible_truncation)]
    const fn next(&mut self) -> u64 {
        // copied from https://github.com/smol-rs/fastrand
        // Constants for WyRand taken from: https://github.com/wangyi-fudan/wyhash/blob/master/wyhash.h#L151
        const WY_CONST_0: u64 = 0x2d35_8dcc_aa6c_78a5;
        const WY_CONST_1: u64 = 0x8bb8_4b93_962e_acc9;

        let s = self.0.wrapping_add(WY_CONST_0);
        self.0 = s;
        let t = s as u128 * (s ^ WY_CONST_1) as u128;
        t as u64 ^ (t >> 64) as u64
    }
}

static KEYS: Keys = Keys::generate();

struct Keys {
    side: u64,
    castle: [u64; 16],
    en_passant: [u64; 64],
    piece: [[u64; 64]; 12],
}

impl Keys {
    pub const fn generate() -> Self {
        let mut rng = Rng(0);
        Self {
            side: rng.next(),
            castle: konst::array::from_fn!(|_| rng.next()),
            en_passant: konst::array::from_fn!(|_| rng.next()),
            piece: konst::array::from_fn!(|_| konst::array::from_fn!(|_| rng.next())),
        }
    }
}
