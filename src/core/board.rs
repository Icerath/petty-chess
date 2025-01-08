use core::fmt;
use std::ops::{Index, IndexMut};

use crate::prelude::*;

#[derive(Clone)]
pub struct Board {
    pub active_side: Side,
    pub can_castle: CanCastle,
    pub en_passant_target_square: Option<Square>,
    pub zobrist: Zobrist,
    pub pieces: [u64; 8],
    pub halfmove_clock: u8,
    pub fullmove_counter: u16,
}

pub struct Unmake {
    board: Board,
}

impl Board {
    pub const EMPTY: Self = Self {
        active_side: Side::White,
        can_castle: CanCastle::empty(),
        en_passant_target_square: None,
        halfmove_clock: 0,
        fullmove_counter: 1,
        zobrist: Zobrist::DEFAULT,
        pieces: [0; 8],
    };

    pub fn swap_side(&mut self) {
        self.active_side = !self.active_side;
        self.zobrist.xor_side_to_move();
    }

    /// Inserts a piece into the board's bitboards.
    ///
    /// This will not remove other pieces from this square and
    /// calling/ this when a piece is already present will produce an invalid zobrist hash
    pub fn insert_piece(&mut self, sq: Square, piece: Piece) {
        self[piece.kind()].insert(sq);
        self[piece.side()].insert(sq);
        self.zobrist.xor_piece(sq, piece);
    }

    /// removes a piece from the board's bitboards
    ///
    /// calling this when a piece is not present will produce an invalid zobrist hash
    pub fn remove_piece(&mut self, sq: Square, piece: Piece) {
        self[piece.kind()].remove(sq);
        self[piece.side()].remove(sq);
        self.zobrist.xor_piece(sq, piece);
    }

    pub fn insert_piece_no_zobrist(&mut self, sq: Square, piece: Piece) {
        self[piece.kind()].insert(sq);
        self[piece.side()].insert(sq);
    }

    pub fn remove_piece_no_zobrist(&mut self, sq: Square, piece: Piece) {
        self[piece.kind()].remove(sq);
        self[piece.side()].remove(sq);
    }

    /// inserts a piece at sq if it doesn't exist or removes it if it does exist.
    pub fn xor_piece(&mut self, sq: Square, piece: Piece) {
        self[piece.kind()] ^= sq;
        self[piece.side()] ^= sq;
        self.zobrist.xor_piece(sq, piece);
    }

    /// # Panics
    /// May panic on illegal moves
    #[track_caller]
    pub fn make_move(&mut self, mov: Move) -> Unmake {
        let unmake = Unmake { board: self.clone() };
        let from_piece = self.active_side + self.get_square_kind(mov.from()).unwrap();

        if let Some(sq) = self.en_passant_target_square {
            self.zobrist.xor_en_passant(sq);
        }
        self.en_passant_target_square = None;
        self.zobrist.xor_can_castle(self.can_castle);
        if from_piece.kind() == PieceKind::King {
            match self.active_side {
                Side::White => self.can_castle.remove(CanCastle::BOTH_WHITE),
                Side::Black => self.can_castle.remove(CanCastle::BOTH_BLACK),
            }
        }
        for sq in [mov.from(), mov.to()] {
            match sq {
                Square::A1 => self.can_castle.remove(CanCastle::WHITE_QUEEN_SIDE),
                Square::H1 => self.can_castle.remove(CanCastle::WHITE_KING_SIDE),
                Square::A8 => self.can_castle.remove(CanCastle::BLACK_QUEEN_SIDE),
                Square::H8 => self.can_castle.remove(CanCastle::BLACK_KING_SIDE),
                _ => {}
            }
        }
        self.zobrist.xor_can_castle(self.can_castle);
        if let Some(piece) = self.get_square(mov.to()) {
            self.remove_piece(mov.to(), piece);
        }
        self.remove_piece(mov.from(), from_piece);
        self.insert_piece(mov.to(), from_piece);

        match mov.flags() {
            MoveFlags::Quiet | MoveFlags::Capture => {}
            MoveFlags::EnPassant => {
                let back = mov.to().add_rank(-self.active_side.forward()).unwrap();
                let pawn = !self.active_side + Pawn;
                self.remove_piece(back, pawn);
            }
            MoveFlags::DoublePawnPush => {
                let back = mov.to().add_rank(-self.active_side.forward()).unwrap();
                self.en_passant_target_square = Some(back);
                self.zobrist.xor_en_passant(back);
            }
            MoveFlags::QueenCastle if self.active_side == White => {
                self.remove_piece(Square::A1, WhiteRook);
                self.insert_piece(Square::D1, WhiteRook);
            }
            MoveFlags::QueenCastle => {
                self.remove_piece(Square::A8, BlackRook);
                self.insert_piece(Square::D8, BlackRook);
            }
            MoveFlags::KingCastle if self.active_side == White => {
                self.remove_piece(Square::H1, WhiteRook);
                self.insert_piece(Square::F1, WhiteRook);
            }
            MoveFlags::KingCastle => {
                self.remove_piece(Square::H8, BlackRook);
                self.insert_piece(Square::F8, BlackRook);
            }
            flags if flags.promotion().is_some() => {
                let piece = self.active_side + PieceKind::from(flags.promotion().unwrap());
                self.remove_piece(mov.to(), from_piece);
                self.insert_piece(mov.to(), piece);
            }
            _ => unreachable!("{:?}", mov.flags()),
        }
        self.increment_ply();
        unmake
    }

    pub(crate) fn make_move_no_update(&mut self, mov: Move) -> Unmake {
        let unmake = Unmake { board: self.clone() };
        let from_piece = self.active_side + self.get_square_kind(mov.from()).unwrap();

        if let Some(piece) = self.get_square_kind(mov.to()) {
            self.remove_piece_no_zobrist(mov.to(), !self.active_side + piece);
        }
        self.remove_piece_no_zobrist(mov.from(), from_piece);
        self.insert_piece_no_zobrist(mov.to(), from_piece);

        if mov.flags() == MoveFlags::EnPassant {
            let back = mov.to().add_rank(-self.active_side.forward()).unwrap();
            let pawn = !self.active_side + Pawn;
            self.remove_piece_no_zobrist(back, pawn);
        }
        unmake
    }

    #[must_use]
    pub fn update_flags(&self, mov: Move) -> Move {
        let is_capture = self[!self.active_side].contains(mov.to());
        if let Some(promotion) = mov.flags().promotion() {
            if !is_capture {
                return mov;
            }

            return match promotion {
                Promotion::Knight => mov.with_flags(MoveFlags::KnightPromotionCapture),
                Promotion::Bishop => mov.with_flags(MoveFlags::BishopPromotionCapture),
                Promotion::Rook => mov.with_flags(MoveFlags::RookPromotionCapture),
                Promotion::Queen => mov.with_flags(MoveFlags::QueenPromotionCapture),
            };
        }
        let is_castling = self[King].contains(mov.from()) && mov.file_diff() > 1;

        if is_castling {
            let flags =
                if mov.file_diff() == 2 { MoveFlags::KingCastle } else { MoveFlags::QueenCastle };
            return mov.with_flags(flags);
        }

        if self[Pawn].contains(mov.from()) && mov.rank_diff() > 1 {
            return mov.with_flags(MoveFlags::DoublePawnPush);
        }
        let is_en_passant = self[Pawn].contains(mov.from())
            && mov.file_diff() != 0
            && self[!self.active_side].contains(mov.to());

        if is_en_passant {
            return mov.with_flags(MoveFlags::EnPassant);
        }

        mov.with_flags(if is_capture { MoveFlags::Capture } else { MoveFlags::Quiet })
    }

    pub fn unmake_move(&mut self, unmake: Unmake) {
        *self = unmake.board;
    }

    pub fn make_null_move(&mut self) -> Option<Square> {
        self.increment_ply();
        self.en_passant_target_square.take().inspect(|&sq| self.zobrist.xor_en_passant(sq))
    }

    pub fn unmake_null_move(&mut self, prev_en_passant: Option<Square>) {
        self.decrement_ply();
        self.en_passant_target_square =
            prev_en_passant.inspect(|&sq| self.zobrist.xor_en_passant(sq));
    }

    pub fn increment_ply(&mut self) {
        self.fullmove_counter += self.active_side.is_black() as u16;
        self.halfmove_clock += 1;
        self.swap_side();
    }

    pub fn decrement_ply(&mut self) {
        self.swap_side();
        self.halfmove_clock -= 1;
        self.fullmove_counter -= self.active_side.is_black() as u16;
    }

    #[must_use]
    pub fn side_bitboards(&self, side: Side) -> Pieces {
        let x: [u64; 6] = self.pieces[..6].try_into().unwrap();
        Pieces(x.map(|bitboard| Bitboard(bitboard) & self[side]))
    }

    #[must_use]
    pub fn friendly_bitboards(&self) -> Pieces {
        self.side_bitboards(self.active_side)
    }

    #[must_use]
    pub fn enemy_bitboards(&self) -> Pieces {
        self.side_bitboards(!self.active_side)
    }

    #[must_use]
    pub fn all_pieces(&self) -> Bitboard {
        self[White] | self[Black]
    }

    #[must_use]
    pub fn get_king_square(&self, side: Side) -> Option<Square> {
        self.get(side + King).bitscan()
    }

    #[must_use]
    pub fn active_king(&self) -> Option<Square> {
        self.get_king_square(self.active_side)
    }

    #[must_use]
    pub fn inactive_king(&self) -> Option<Square> {
        self.get_king_square(!self.active_side)
    }
}

impl Board {
    #[must_use]
    pub fn get(&self, piece: Piece) -> Bitboard {
        self[piece.kind()] & self[piece.side()]
    }

    pub fn swap(&mut self, lhs: Square, rhs: Square) {
        let lhs_piece = self.get_square(lhs);
        let rhs_piece = self.get_square(rhs);
        if let Some(piece) = lhs_piece {
            self.remove_piece(lhs, piece);
            self.insert_piece(rhs, piece);
        }
        if let Some(piece) = rhs_piece {
            self.remove_piece(rhs, piece);
            self.insert_piece(lhs, piece);
        }
    }

    #[must_use]
    pub fn is_piece_at(&self, sq: Square) -> bool {
        self.all_pieces().contains(sq)
    }

    #[must_use]
    pub fn is_side(&self, sq: Square, side: Side) -> bool {
        self[side].contains(sq)
    }

    #[must_use]
    pub fn get_square(&self, square: Square) -> Option<Piece> {
        let side = if self[White].contains(square) { White } else { Black };
        let kind = self.get_square_kind(square)?;
        Some(side + kind)
    }

    #[must_use]
    pub fn get_square_kind(&self, square: Square) -> Option<PieceKind> {
        PieceKind::ALL.into_iter().find(|&kind| self[kind].contains(square))
    }

    #[must_use]
    pub fn in_check(&self) -> bool {
        !self.gen_checkers(self.active_side).is_empty()
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_fen())
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::start_pos()
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Pieces(pub [Bitboard; 6]);

impl Index<PieceKind> for Pieces {
    type Output = Bitboard;

    fn index(&self, kind: PieceKind) -> &Self::Output {
        &self.0[kind as usize]
    }
}

impl IndexMut<PieceKind> for Pieces {
    fn index_mut(&mut self, kind: PieceKind) -> &mut Self::Output {
        &mut self.0[kind as usize]
    }
}
impl Index<PieceKind> for Board {
    type Output = Bitboard;

    fn index(&self, kind: PieceKind) -> &Self::Output {
        Bitboard::from_ref(&self.pieces[kind as usize])
    }
}

impl IndexMut<PieceKind> for Board {
    fn index_mut(&mut self, kind: PieceKind) -> &mut Self::Output {
        Bitboard::from_mut(&mut self.pieces[kind as usize])
    }
}

impl Index<Side> for Board {
    type Output = Bitboard;

    fn index(&self, side: Side) -> &Self::Output {
        Bitboard::from_ref(&self.pieces[side as usize + 6])
    }
}

impl IndexMut<Side> for Board {
    fn index_mut(&mut self, side: Side) -> &mut Self::Output {
        Bitboard::from_mut(&mut self.pieces[side as usize + 6])
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Board {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_fen().serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Board {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let fen: &str = serde::Deserialize::deserialize(deserializer)?;
        Self::from_fen(fen)
            .ok_or_else(|| serde::de::Error::custom(format_args!("Unexpected: {fen:?}")))
    }
}
