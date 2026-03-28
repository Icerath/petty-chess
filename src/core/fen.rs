use std::fmt::Write;

use crate::prelude::*;

pub const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
pub const KIWIPETE: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -";
#[cfg(test)]
pub const PERFT_POSITION_3: &str = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -";
#[cfg(test)]
pub const PERFT_POSITION_4: &str =
    "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
#[cfg(test)]
pub const PERFT_POSITION_5: &str = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
#[cfg(test)]
pub const EN_PASSANT_TEST: &str = "rnbqkbnr/pppppppp/8/8/3Pp3/8/PPP2PPP/RNBQKBNR b KQkq d3 0 2";

#[expect(clippy::missing_panics_doc)]
impl Board {
    #[must_use]
    pub fn start_pos() -> Self {
        Self::from_fen(STARTING_FEN).expect("Starting FEN should be valid FEN")
    }

    #[must_use]
    pub fn kiwipete() -> Self {
        Self::from_fen(KIWIPETE).expect("Kiwipete should be valid FEN")
    }

    #[must_use]
    #[cfg(test)]
    pub fn perft_position_3() -> Self {
        Self::from_fen(PERFT_POSITION_3).expect("Should be valid FEN")
    }

    #[must_use]
    #[cfg(test)]
    pub fn perft_position_4() -> Self {
        Self::from_fen(PERFT_POSITION_4).expect("Should be valid FEN")
    }

    #[must_use]
    #[cfg(test)]
    pub fn perft_position_5() -> Self {
        Self::from_fen(PERFT_POSITION_5).expect("Should be valid FEN")
    }

    pub fn to_fen_into(&self, buf: &mut String) {
        let mut prev = None::<Square>;
        for sq in Square::ALL {
            if let Some(piece) = self.get_square(sq.flip()) {
                if let Some(prev) = prev {
                    if let Some(dif @ 1..) =
                        (sq.file() as u8).checked_sub((prev.file() as u8 + 1) % 8)
                    {
                        buf.push((dif + b'0') as char);
                    }
                } else if sq.file() as u8 != 0 {
                    buf.push((sq.file() as u8 + b'0') as char);
                }
                buf.push(piece.symbol());
                prev = Some(sq);
            }
            if sq != Square::H8 && sq.file() as u8 == 7 {
                if !self.is_piece_at(sq.flip()) {
                    if let Some(prev) = prev {
                        if let dif @ 1.. = 8 - (prev.file() as u8 + 1) % 8 {
                            buf.push((dif + b'0') as char);
                        }
                    } else {
                        buf.push('8');
                    }
                }
                buf.push('/');
                prev = Some(sq);
            }
        }

        if !self.is_piece_at(Square::H1)
            && let dif @ 1.. = 8 - (prev.unwrap().file() as u8 + 1) % 8
        {
            buf.push((dif + b'0') as char);
        }

        buf.push(' ');
        buf.push(self.active_side.symbol());
        write!(buf, " {}", self.can_castle).expect("Writing to a string should not fail");

        buf.push(' ');
        match self.en_passant_target_square {
            Some(sq) => buf.push_str(sq.algebraic()),
            _ => buf.push('-'),
        }

        write!(buf, " {} {}", self.halfmove_clock, self.fullmove_counter)
            .expect("Writing to a string should not fail");
    }

    #[must_use]
    pub fn to_fen(&self) -> String {
        let mut builder = String::new();
        self.to_fen_into(&mut builder);
        builder
    }

    #[must_use]
    pub fn from_fen<S: AsRef<[u8]>>(fen: S) -> Option<Self> {
        let mut fields = fen.as_ref().split(|&b| b == b' ');

        let mut board = parse_pieces(fields.next()?)?;
        let active_side = match fields.next()? {
            b"w" => White,
            b"b" => Black,
            _ => return None,
        };
        let can_castle = parse_can_castle(fields.next()?)?;
        let en_passant_target_square = parse_en_passant(fields.next()?)?;
        let halfmove_clock = fields.next().and_then(atoi::atoi).unwrap_or(0);
        let fullmove_counter = fields.next().and_then(atoi::atoi).unwrap_or(1);

        if active_side == Black {
            board.swap_side();
        }
        board.can_castle = can_castle;
        board.zobrist.xor_can_castle(can_castle);
        board.en_passant_target_square = en_passant_target_square;
        board.halfmove_clock = halfmove_clock;
        board.fullmove_counter = fullmove_counter;
        if let Some(en_passant) = en_passant_target_square {
            board.zobrist.xor_en_passant(en_passant);
        }
        Some(board)
    }
}

fn parse_pieces(fen: &[u8]) -> Option<Board> {
    let mut board = Board::EMPTY;
    let mut rank = 7;
    let mut file = 0;

    for c in fen {
        let kind = match c.to_ascii_lowercase() {
            b'1'..=b'9' => {
                file += c - b'0';
                continue;
            }
            b'/' => {
                file = 0;
                rank -= 1;
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
        let side = if c.is_ascii_uppercase() { White } else { Black };
        let sq = Square::new(Rank::from_int(rank).unwrap(), File::from_int(file).unwrap());
        board.insert_piece(sq, side + kind);
        file += 1;
    }
    Some(board)
}

fn parse_can_castle(fen: &[u8]) -> Option<CanCastle> {
    let mut can_castle = CanCastle::empty();

    for byte in fen {
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

#[expect(clippy::option_option)]
fn parse_en_passant(fen: &[u8]) -> Option<Option<Square>> {
    if fen == b"-" {
        return Some(None);
    }
    let bstr = fen.try_into().ok()?;
    Square::from_bstr(bstr).map(Some)
}

#[test]
fn test_fen_parsing() {
    let board = Board::from_fen(STARTING_FEN).expect("Failed to parse starting fen");
    assert_eq!(board.get_square(Square::E1), Some(WhiteKing));
    assert_eq!(board.to_fen(), STARTING_FEN);

    for fen in [KIWIPETE, PERFT_POSITION_3, PERFT_POSITION_4, PERFT_POSITION_5] {
        let board = Board::from_fen(fen).expect("Failed to parse fen string");
        let mut board_fen = board.to_fen();
        if fen.split_whitespace().count() == 4 {
            // Todo - account for double digit counters.
            board_fen.truncate(board_fen.len() - 4);
        }
        assert_eq!(board_fen, fen);
    }
}

#[test]
fn test_can_castle() {
    assert_eq!(parse_can_castle(b"Kkq"), Some(CanCastle::WHITE_KING_SIDE | CanCastle::BOTH_BLACK));
}

#[test]
fn test_fen_en_passant() {
    assert_eq!(parse_en_passant(b"g6"), Some(Some(Square::G6)));
    assert_eq!(parse_en_passant(b"-"), Some(None));
    assert_eq!(parse_en_passant(b"1a"), None);

    let board = Board::from_fen(EN_PASSANT_TEST).unwrap();
    assert_eq!(board.en_passant_target_square, Some(Square::D3));
}
