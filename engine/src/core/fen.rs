use crate::prelude::*;
use std::fmt::Write;

pub const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

impl Board {
    pub fn start_pos() -> Self {
        Self::from_fen(STARTING_FEN).expect("Starting FEN should be valid FEN")
    }
    pub fn to_fen_into(&self, buf: &mut String) {
        let mut prev = None::<Pos>;
        for pos in (0..64).map(Pos) {
            if let Some(piece) = self[pos] {
                if let Some(prev) = prev {
                    if let Some(dif @ 1..) = pos.col().0.checked_sub((prev.col().0 + 1) % 8) {
                        buf.push((dif + b'0') as char);
                    }
                }
                buf.push(piece.symbol());
                prev = Some(pos);
            }
            if pos != Pos(63) && pos.col().0 == 7 {
                if self[pos].is_none() {
                    if let Some(prev) = prev {
                        if let dif @ 1.. = 8 - (prev.col().0 + 1) % 8 {
                            buf.push((dif + b'0') as char);
                        }
                    }
                }
                buf.push('/');
                prev = Some(Pos(pos.0));
            }
        }
        buf.push(' ');

        buf.push(if self.active_colour.is_white() { 'w' } else { 'b' });
        buf.push(' ');

        if self.can_castle.contains(CanCastle::WHITE_KING_SIDE) {
            buf.push('K');
        }
        if self.can_castle.contains(CanCastle::WHITE_QUEEN_SIDE) {
            buf.push('Q');
        }
        if self.can_castle.contains(CanCastle::BLACK_KING_SIDE) {
            buf.push('k');
        }
        if self.can_castle.contains(CanCastle::BLACK_QUEEN_SIDE) {
            buf.push('q');
        }
        if self.can_castle.is_empty() {
            buf.push('-');
        }
        buf.push(' ');

        match self.en_passant_target_square {
            Some(pos) => buf.push_str(pos.algebraic()),
            _ => buf.push('-'),
        }
        buf.push(' ');

        let _ = write!(buf, "{} ", self.halfmove_clock);
        let _ = write!(buf, "{}", self.fullmove_counter);
    }
    pub fn to_fen(&self) -> String {
        let mut builder = String::new();
        self.to_fen_into(&mut builder);
        builder
    }
    pub fn from_fen(fen: &str) -> Option<Self> {
        let mut fields = fen.split(' ');
        let pieces = parse_pieces(fields.next()?)?;
        let active_colour = match fields.next()? {
            "w" => White,
            "b" => Black,
            _ => return None,
        };
        let can_castle = parse_can_castle(fields.next()?)?;
        let en_passant_target_square = parse_en_passant(fields.next()?)?;
        let halfmove_clock: u8 = fields.next()?.parse().ok()?;
        let fullmove_counter: u16 = fields.next()?.parse().ok()?;

        Some(Self {
            pieces,
            active_colour,
            can_castle,
            en_passant_target_square,
            halfmove_clock,
            fullmove_counter,
        })
    }
}

fn parse_pieces(fen: &str) -> Option<[Option<Piece>; 64]> {
    let mut row = 0;
    let mut col = 0;

    let mut pieces = [None; 64];
    for c in fen.bytes() {
        let kind = match c.to_ascii_lowercase() {
            b'1'..=b'9' => {
                col += c - b'0';
                continue;
            }
            b'/' => {
                col = 0;
                row += 1;
                continue;
            }
            b'p' => Pawn,
            b'n' => Knight,
            b'b' => Bishop,
            b'r' => Rook,
            b'q' => Queen,
            b'k' => King,
            _ => return None,
        };
        let colour = if c.is_ascii_uppercase() { White } else { Black };
        let pos = Pos::new(Row(row), Col(col));
        pieces[pos.0 as usize] = Some(Piece::new(kind, colour));
        col += 1;
    }

    Some(pieces)
}

fn parse_can_castle(fen: &str) -> Option<CanCastle> {
    let mut can_castle = CanCastle::empty();

    for byte in fen.as_bytes() {
        match byte {
            b'-' => return Some(can_castle),
            b'K' => can_castle |= CanCastle::WHITE_KING_SIDE,
            b'Q' => can_castle |= CanCastle::WHITE_QUEEN_SIDE,
            b'k' => can_castle |= CanCastle::BLACK_KING_SIDE,
            b'q' => can_castle |= CanCastle::BLACK_QUEEN_SIDE,
            _ => return None,
        }
    }

    Some(can_castle)
}

fn parse_en_passant(fen: &str) -> Option<Option<Pos>> {
    if fen == "-" {
        return Some(None);
    }
    fen.parse().map(Some).ok()
}

#[test]
fn test_fen_parsing() {
    let board = Board::from_fen(STARTING_FEN).expect("Failed to parse starting fen");
    assert_eq!(board.to_fen(), STARTING_FEN);

    let fen = "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2";
    let board = Board::from_fen(fen).expect("Failed to parse fen string");
    assert_eq!(board.to_fen(), fen);

    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    let board = Board::from_fen(fen).expect("Failed to parse fen string");
    assert_eq!(board.to_fen(), fen);
}
