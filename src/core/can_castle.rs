bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct CanCastle: u8 {
        const WHITE_KING_SIDE = 0b0001;
        const WHITE_QUEEN_SIDE = 0b0010;
        const BLACK_KING_SIDE = 0b0100;
        const BLACK_QUEEN_SIDE = 0b1000;

        const BOTH_WHITE = Self::WHITE_KING_SIDE.bits() | Self::WHITE_QUEEN_SIDE.bits();
        const BOTH_BLACK = Self::BLACK_KING_SIDE.bits() | Self::BLACK_QUEEN_SIDE.bits();
    }

}

#[test]
fn test_can_castle() {
    let mut can_castle = CanCastle::all();

    assert!(can_castle.contains(CanCastle::all()));
    assert!(can_castle.contains(CanCastle::BOTH_WHITE));
    assert!(can_castle.contains(CanCastle::BOTH_BLACK));
    assert!(can_castle.contains(CanCastle::WHITE_KING_SIDE));
    assert!(can_castle.contains(CanCastle::BLACK_KING_SIDE));
    assert!(can_castle.contains(CanCastle::WHITE_QUEEN_SIDE));
    assert!(can_castle.contains(CanCastle::BLACK_QUEEN_SIDE));

    can_castle.remove(CanCastle::WHITE_KING_SIDE);
    assert!(!can_castle.contains(CanCastle::WHITE_KING_SIDE));

    can_castle.remove(CanCastle::BLACK_KING_SIDE);
    assert!(!can_castle.contains(CanCastle::BLACK_KING_SIDE));

    can_castle.remove(CanCastle::WHITE_QUEEN_SIDE);
    assert!(!can_castle.contains(CanCastle::WHITE_QUEEN_SIDE));

    can_castle.remove(CanCastle::BLACK_QUEEN_SIDE);
    assert!(!can_castle.contains(CanCastle::BLACK_QUEEN_SIDE));

    let mut can_castle = CanCastle::BOTH_BLACK;
    can_castle.remove(CanCastle::BOTH_BLACK);
    assert!(!can_castle.contains(CanCastle::BOTH_BLACK));
}
