bitflags::bitflags! {
    pub struct CanCastle: u8 {
        const WHITE_KING_SIDE = 0b0001;
        const WHITE_QUEEN_SIDE = 0b0010;
        const BLACK_KING_SIDE = 0b0100;
        const BLACK_QUEEN_SIDE = 0b0101;

        const BOTH_WHITE = Self::WHITE_KING_SIDE.bits() | Self::WHITE_QUEEN_SIDE.bits();
        const BOTH_BLACK = Self::BLACK_KING_SIDE.bits() | Self::BLACK_KING_SIDE.bits();
        const ALL = Self::BOTH_WHITE.bits() | Self::BOTH_BLACK.bits();
    }

}
