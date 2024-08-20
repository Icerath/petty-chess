use core::fmt;
use std::ops::{Index, IndexMut};

use crate::prelude::*;

#[derive(Clone)]
pub struct Board {
    pub active_side: Side,
    pub can_castle: CanCastle,
    pub en_passant_target_square: Option<Square>,
    pub zobrist: Zobrist,
    pub piece_bitboards: Pieces,
    pub side_pieces: SidePieces,
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
        piece_bitboards: Pieces([Bitboard::EMPTY; 6]),
        side_pieces: SidePieces([Bitboard::EMPTY; 2]),
    };
    pub fn swap_side(&mut self) {
        self.active_side = !self.active_side;
        self.zobrist.xor_side_to_move();
    }
    /// Inserts a piece into the board's bitboards.
    ///
    /// This will not remove other pieces from this square and
    /// calling/ this when a piece is already present with produce an invalid zobrist hash
    pub fn insert_piece(&mut self, sq: Square, piece: Piece) {
        self[piece.kind()].insert(sq);
        self[piece.side()].insert(sq);
        self.zobrist.xor_piece(sq, piece);
    }
    /// removes a piece from the board's bitboards
    ///
    /// calling this when a piece is not present with produce an invalid zobrist hash
    pub fn remove_piece(&mut self, sq: Square, piece: Piece) {
        self[piece.kind()].remove(sq);
        self[piece.side()].remove(sq);
        self.zobrist.xor_piece(sq, piece);
    }
    /// inserts a piece at sq if it doesn't exist or removes it if it does exist.
    pub fn xor_piece(&mut self, sq: Square, piece: Piece) {
        self[piece.kind()] ^= sq;
        self[piece.side()] ^= sq;
        self.zobrist.xor_piece(sq, piece);
    }
    pub fn make_move(&mut self, mov: Move) -> Unmake {
        let unmake = Unmake { board: self.clone() };

        let from_piece = self.get_square(mov.from()).unwrap();

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
            MoveFlags::QueenCastle if self.active_side == White => self.swap(Square::A1, Square::D1),
            MoveFlags::QueenCastle => self.swap(Square::A8, Square::D8),
            MoveFlags::KingCastle if self.active_side == White => self.swap(Square::F1, Square::H1),
            MoveFlags::KingCastle => self.swap(Square::F8, Square::H8),
            MoveFlags::DoublePawnPush => {
                let back = mov.to().add_rank(-self.active_side.forward()).unwrap();
                self.en_passant_target_square = Some(back);
                self.zobrist.xor_en_passant(back);
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
    pub fn unmake_move(&mut self, unmake: Unmake) {
        *self = unmake.board;
    }
    pub fn make_null_move(&mut self) -> Option<Square> {
        self.increment_ply();
        self.en_passant_target_square.take()
    }
    pub fn unmake_null_move(&mut self, prev_en_passant: Option<Square>) {
        self.en_passant_target_square = prev_en_passant;
        self.decrement_ply();
    }
    #[inline]
    pub fn increment_ply(&mut self) {
        self.zobrist.xor_side_to_move();
        if self.black_to_play() {
            self.fullmove_counter += 1;
        }
        self.halfmove_clock += 1;
        self.active_side = !self.active_side;
    }
    #[inline]
    pub fn decrement_ply(&mut self) {
        self.active_side = !self.active_side;
        self.halfmove_clock -= 1;
        if self.black_to_play() {
            self.fullmove_counter -= 1;
        }
        self.zobrist.xor_side_to_move();
    }
    #[inline]
    #[must_use]
    pub fn white_to_play(&self) -> bool {
        self.active_side.is_white()
    }
    #[inline]
    #[must_use]
    pub fn black_to_play(&self) -> bool {
        self.active_side.is_black()
    }
    #[must_use]
    #[inline]
    pub fn side_bitboards(&self, side: Side) -> Pieces {
        self.piece_bitboards.map(|bitboard| bitboard & self[side])
    }
    #[must_use]
    #[inline]
    pub fn friendly_bitboards(&self) -> Pieces {
        self.side_bitboards(self.active_side)
    }
    #[must_use]
    #[inline]
    pub fn enemy_bitboards(&self) -> Pieces {
        self.side_bitboards(!self.active_side)
    }
    #[must_use]
    #[inline]
    pub fn all_pieces(&self) -> Bitboard {
        self[White] | self[Black]
    }
    #[must_use]
    #[inline]
    pub fn get_king_square(&self, side: Side) -> Option<Square> {
        self.get(side + King).bitscan()
    }
    #[must_use]
    #[inline]
    pub fn active_king(&self) -> Option<Square> {
        self.get_king_square(self.active_side)
    }
    #[must_use]
    #[inline]
    pub fn inactive_king(&self) -> Option<Square> {
        self.get_king_square(!self.active_side)
    }
}

impl Board {
    #[must_use]
    #[inline]
    pub fn get(&self, piece: Piece) -> Bitboard {
        self.piece_bitboards[piece.kind()] & self.side_pieces[piece.side()]
    }
    #[inline]
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
    #[inline]
    pub fn for_each_piece(&self, mut f: impl FnMut(Square, Piece)) {
        for piece in Piece::ALL {
            self.get(piece).for_each(|sq| f(sq, piece));
        }
    }
    #[inline]
    #[must_use]
    pub fn is_piece_at(&self, sq: Square) -> bool {
        self.all_pieces().contains(sq)
    }
    #[inline]
    #[must_use]
    pub fn is_side(&self, sq: Square, side: Side) -> bool {
        self[side].contains(sq)
    }
    #[inline]
    #[must_use]
    pub fn get_square(&self, square: Square) -> Option<Piece> {
        let side = if self[White].contains(square) { White } else { Black };
        let kind = self.get_square_kind(square)?;
        Some(side + kind)
    }
    #[inline]
    #[must_use]
    pub fn get_square_kind(&self, square: Square) -> Option<PieceKind> {
        PieceKind::ALL.into_iter().find(|&kind| self[kind].contains(square))
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
pub struct Pieces([Bitboard; 6]);

impl Pieces {
    #[inline]
    #[must_use]
    pub fn map(&self, f: impl FnMut(Bitboard) -> Bitboard) -> Self {
        Self(self.0.map(f))
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct SidePieces([Bitboard; 2]);

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

impl Index<Side> for SidePieces {
    type Output = Bitboard;
    fn index(&self, side: Side) -> &Self::Output {
        &self.0[side as usize]
    }
}

impl IndexMut<Side> for SidePieces {
    fn index_mut(&mut self, side: Side) -> &mut Self::Output {
        &mut self.0[side as usize]
    }
}

impl Index<PieceKind> for Board {
    type Output = Bitboard;
    fn index(&self, index: PieceKind) -> &Self::Output {
        &self.piece_bitboards[index]
    }
}

impl IndexMut<PieceKind> for Board {
    fn index_mut(&mut self, kind: PieceKind) -> &mut Self::Output {
        &mut self.piece_bitboards[kind]
    }
}

impl Index<Side> for Board {
    type Output = Bitboard;
    fn index(&self, side: Side) -> &Self::Output {
        &self.side_pieces[side]
    }
}

impl IndexMut<Side> for Board {
    fn index_mut(&mut self, side: Side) -> &mut Self::Output {
        &mut self.side_pieces[side]
    }
}
