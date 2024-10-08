use std::sync::OnceLock;

use movegen::{DIRECTION_OFFSETS, NUM_SQUARES_TO_EDGE};
use rand::prelude::*;

use crate::prelude::*;

const ROOK: usize = 4096;
const BISHOP: usize = 512;

pub struct Magic {
    rook_tables: [Box<SquareTables<ROOK>>; 64],
    bishop_tables: [Box<SquareTables<BISHOP>>; 64],
}

static MAGIC: OnceLock<Magic> = OnceLock::new();

impl Magic {
    #[inline]
    pub fn get() -> &'static Magic {
        MAGIC.get_or_init(Self::preinit)
    }
    #[inline]
    pub fn get_init() -> &'static Magic {
        MAGIC.get_or_init(Self::init)
    }
    #[must_use]
    #[inline]
    pub fn rook_attacks(&self, sq: Square, occupancy: Bitboard) -> Bitboard {
        self.rook_tables[sq].get_attacks(occupancy)
    }
    #[must_use]
    #[inline]
    pub fn bishop_attacks(&self, sq: Square, occupancy: Bitboard) -> Bitboard {
        self.bishop_tables[sq].get_attacks(occupancy)
    }
    #[must_use]
    #[inline]
    pub fn queen_attacks(&self, sq: Square, occupancy: Bitboard) -> Bitboard {
        self.bishop_tables[sq].get_attacks(occupancy) | self.rook_tables[sq].get_attacks(occupancy)
    }
    #[allow(unused)]
    fn init() -> Magic {
        Self {
            rook_tables: std::array::from_fn(|i| {
                Box::new(SquareTables::init(Square::try_from(i).unwrap()).unwrap())
            }),
            bishop_tables: std::array::from_fn(|i| {
                Box::new(SquareTables::init(Square::try_from(i).unwrap()).unwrap())
            }),
        }
    }
    #[allow(unused)]
    fn preinit() -> Magic {
        Self {
            rook_tables: std::array::from_fn(|i| Box::new(SquareTables::preinit(Square::try_from(i).unwrap()))),
            bishop_tables: std::array::from_fn(|i| {
                Box::new(SquareTables::preinit(Square::try_from(i).unwrap()))
            }),
        }
    }
}

struct SquareTables<const PIECE: usize> {
    mask: u64,
    shift: u32,
    attacks: [u64; PIECE],
    magic: u64,
}

impl<const PIECE: usize> SquareTables<PIECE> {
    fn preinit(sq: Square) -> Self {
        let mask = Self::mask(sq);
        let magic = if PIECE == BISHOP { BISHOP_MAGICS[sq] } else { ROOK_MAGICS[sq] };
        let mut attacks: [u64; PIECE] = [0; PIECE];

        for i in 0..1 << mask.count_ones() {
            let occupancy = index_to_bb(i, mask);
            let index = (occupancy.0.wrapping_mul(magic) >> (mask.count_zeros())) as usize;
            attacks[index] = Self::attacks(sq, occupancy);
        }
        Self { mask, shift: mask.count_zeros(), attacks, magic }
    }

    fn init(sq: Square) -> Result<Self, ()> {
        let mask = Self::mask(sq);
        let occupancies: [Bitboard; PIECE] = std::array::from_fn(|i| index_to_bb(i, mask));
        let attacks = occupancies.map(|occupancy| Self::attacks(sq, occupancy));
        let mut used = [0; PIECE];

        'trials: for _ in 0..100_000_000 {
            used.fill(0);
            let magic = random_u64_fewbits();

            for (i, occupancy) in occupancies.into_iter().enumerate() {
                let index = (occupancy.0.wrapping_mul(magic) >> (mask.count_zeros())) as usize;
                if used[index] == 0 {
                    used[index] = attacks[i];
                } else if used[index] != attacks[i] {
                    continue 'trials;
                }
            }
            return Ok(Self { mask, magic, shift: mask.count_zeros(), attacks: used });
        }
        Err(())
    }
    #[inline]
    fn get_attacks(&self, mut occupancy: Bitboard) -> Bitboard {
        occupancy.0 &= self.mask;
        let index = (occupancy.0.wrapping_mul(self.magic) >> self.shift) as usize;
        Bitboard(self.attacks[index])
    }
    #[allow(clippy::needless_range_loop)]
    fn mask(sq: Square) -> u64 {
        let mut result = Bitboard(0);
        let start = if PIECE == BISHOP { 4 } else { 0 };
        let end = if PIECE == ROOK { 4 } else { 8 };

        for direction_index in start..end {
            for n in 1..NUM_SQUARES_TO_EDGE[sq][direction_index] {
                let target_square =
                    Square::try_from(i8::from(sq) + DIRECTION_OFFSETS[direction_index] * n).unwrap();
                result.insert(target_square);
            }
        }
        result.0
    }
    #[allow(clippy::needless_range_loop)]
    fn attacks(sq: Square, occupancy: Bitboard) -> u64 {
        let mut result = Bitboard(0);
        let start = if PIECE == BISHOP { 4 } else { 0 };
        let end = if PIECE == ROOK { 4 } else { 8 };

        for direction_index in start..end {
            for n in 1..=NUM_SQUARES_TO_EDGE[sq][direction_index] {
                let target_square =
                    Square::try_from(i8::from(sq) + DIRECTION_OFFSETS[direction_index] * n).unwrap();
                result.insert(target_square);
                if occupancy.contains(target_square) {
                    break;
                };
            }
        }
        result.0
    }
}

fn index_to_bb(index: usize, mut m: u64) -> Bitboard {
    let mut result = Bitboard::EMPTY;
    for i in 0..m.count_ones() {
        let j = pop_1st_bit(&mut m);
        if (index & (1 << i)) > 0 {
            result.0 |= 1 << j;
        };
    }
    result
}

fn pop_1st_bit(bb: &mut u64) -> u32 {
    let bit = bb.trailing_zeros();
    *bb &= *bb - 1;
    bit
}

fn random_u64_fewbits() -> u64 {
    let mut rng = rand::thread_rng();
    rng.gen::<u64>() & rng.gen::<u64>() & rng.gen::<u64>()
}

#[allow(clippy::unreadable_literal)]
const ROOK_MAGICS: [u64; 64] = [
    2377919024374743074,
    1188968168765382656,
    180179170683260928,
    3530832003549106176,
    72066392345739280,
    2341874007405581312,
    108096286686708224,
    3530822393508015360,
    13898249188103159936,
    432486439180058632,
    187040190812914304,
    9223653550488686592,
    9271082595230189568,
    144678258993008808,
    54606162678644744,
    9313584796032567424,
    153545149688594432,
    72268975158857728,
    153122937708216448,
    76562843067109504,
    9713635542730752,
    288371663462994432,
    648522744522706946,
    1170982082643002373,
    328206368456712,
    576742369018249344,
    5863862638845362304,
    11529496628420411552,
    2256200008204416,
    9260527834366083136,
    4612257919093048720,
    581245839792947330,
    9944581296745350274,
    1441292893133217808,
    5503574668663074816,
    576480139819354176,
    874138166737503488,
    5207295667487744,
    613617717039072520,
    81065070351618340,
    2612088472151818244,
    9008642365865984,
    73465656117362754,
    4611976710408241168,
    582094650018398336,
    6192519348092952,
    1168265969800,
    414531584001,
    72127963327369344,
    292734250659088448,
    2308246953954115712,
    8798508974208,
    4613937835425267776,
    373803169366442112,
    77203927100113920,
    2269942896460288,
    37013959978537229,
    1298197852127756547,
    648588784342532226,
    1162504989956901121,
    72621162603004962,
    1225542205633464578,
    2995459247057735692,
    3458764790460285954,
];

#[allow(clippy::unreadable_literal)]
const BISHOP_MAGICS: [u64; 64] = [
    4629788936221237376,
    2630667365737627904,
    301745643146249216,
    146377983542853760,
    72356936363802688,
    9578954042115848,
    1171050261051867146,
    360572745330722816,
    1152941364754252928,
    585574063221571616,
    1495476029950131,
    1441165126479642624,
    2819160841126400,
    36038710340764704,
    4400899690512,
    612489832924710400,
    54047903085441536,
    1166449904533438720,
    289994164610204160,
    146437511319660552,
    13835199084838780931,
    20548500350042373,
    9237463382306132992,
    281483575191812,
    723144751865397508,
    4512672914080000,
    22605976381170752,
    721139440222765186,
    9872735908175822856,
    11601698172774391808,
    18650191712485888,
    1829725894937601,
    617151512991240740,
    3461027317548582912,
    2342162111664292416,
    720611159111172688,
    2251955506512128,
    9078137090539584,
    2451093494853402880,
    36675310934041088,
    36187161724199044,
    283330671019042,
    27656291396620304,
    141842502910208,
    1129207333692416,
    9585359874603417856,
    1156316813711116352,
    1130300201616416,
    648594281507852288,
    1729665003216438048,
    4611692066949858306,
    2324534708522191872,
    288230522250067968,
    576542322339217408,
    9029241573474848,
    289357384227293217,
    2323285517861888,
    1442014208320606208,
    9223380833686128640,
    4614079664911063040,
    594475151486157056,
    586030935905419776,
    2306511654037291280,
    4902326610100756512,
];

#[test]
fn test_magic_init() {
    Magic::get_init();
    super::perft::tests::perft_kiwi();
}
