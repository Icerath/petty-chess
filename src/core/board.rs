use core::fmt;
use std::ops::{Deref, DerefMut, Index, IndexMut};

use crate::prelude::*;

#[derive(Clone)]
pub struct Board {
    pub pieces: [Option<Piece>; 64],
    pub active_side: Side,
    pub can_castle: CanCastle,
    pub en_passant_target_square: Option<Square>,
    pub halfmove_clock: u8,
    pub fullmove_counter: u16,
    pub cached: Cached,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Cached {
    pub zobrist: Zobrist,
    pub piece_bitboards: Pieces,
    pub side_pieces: SidePieces,
}

pub struct Unmake {
    cached: Cached,
    mov: Move,
    captured_piece: Option<Piece>,
    can_castle: CanCastle,
    en_passant_target_square: Option<Square>,
}

impl Board {
    pub fn create_cache(&mut self) {
        self.zobrist = Zobrist::default();
        if self.black_to_play() {
            self.zobrist.xor_side_to_move();
        }
        self.cached.zobrist.xor_can_castle(self.can_castle);
        if let Some(sq) = self.en_passant_target_square {
            self.cached.zobrist.xor_en_passant(sq);
        }
        for (sq, piece) in self.piece_squares() {
            self.insert_piece(sq, piece);
            self.zobrist.xor_piece(sq, piece);
        }
    }
    // inserts a piece into the board's bitboards
    pub fn insert_piece(&mut self, sq: Square, piece: Piece) {
        self[piece.kind()].insert(sq);
        self[piece.side()].insert(sq);
    }
    // inserts a piece from the board's bitboards
    pub fn remove_piece(&mut self, sq: Square, piece: Piece) {
        self[piece.kind()].remove(sq);
        self[piece.side()].remove(sq);
    }
    pub fn make_move(&mut self, mov: Move) -> Unmake {
        let from_piece = self[mov.from()].unwrap();

        let mut unmake = Unmake {
            cached: self.cached.clone(),
            mov,
            captured_piece: self[mov.to()],
            can_castle: self.can_castle,
            en_passant_target_square: self.en_passant_target_square,
        };
        if let Some(sq) = self.en_passant_target_square {
            self.zobrist.xor_en_passant(sq);
        }
        self.en_passant_target_square = None;
        self.cached.zobrist.xor_can_castle(self.can_castle);
        if from_piece.kind() == PieceKind::King {
            if self.white_to_play() {
                self.can_castle.remove(CanCastle::BOTH_WHITE);
            } else {
                self.can_castle.remove(CanCastle::BOTH_BLACK);
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

        self.cached.zobrist.xor_can_castle(self.can_castle);
        self.zobrist.xor_piece(mov.from(), from_piece);
        self.zobrist.xor_piece(mov.to(), from_piece);
        if let Some(piece) = self[mov.to()] {
            self.zobrist.xor_piece(mov.to(), piece);
            self.remove_piece(mov.to(), piece);
        }
        self.remove_piece(mov.from(), from_piece);
        self.insert_piece(mov.to(), from_piece);

        self[mov.from()] = None;
        self[mov.to()] = Some(from_piece);

        match mov.flags() {
            MoveFlags::EnPassant => {
                let back = mov.to().add_rank(-self.active_side.forward()).unwrap();
                let pawn = self[back].unwrap();
                unmake.captured_piece = Some(pawn);
                self.cached.zobrist.xor_piece(back, pawn);
                self.remove_piece(back, pawn);
                self[back] = None;
            }
            MoveFlags::QueenCastle if self.white_to_play() => {
                self.swap(Square::A1, Square::D1);
            }
            MoveFlags::QueenCastle => {
                self.swap(Square::A8, Square::D8);
            }
            MoveFlags::KingCastle if self.white_to_play() => {
                self.swap(Square::F1, Square::H1);
            }
            MoveFlags::KingCastle => {
                self.swap(Square::F8, Square::H8);
            }
            MoveFlags::DoublePawnPush => {
                let back = mov.to().add_rank(-self.active_side.forward()).unwrap();
                self.en_passant_target_square = Some(back);
                self.zobrist.xor_en_passant(back);
            }
            flags if flags.promotion().is_some() => {
                let piece = self.active_side + PieceKind::from(flags.promotion().unwrap());
                self[mov.to()] = Some(piece);
                self.zobrist.xor_piece(mov.to(), from_piece);
                self.zobrist.xor_piece(mov.to(), piece);
                self.remove_piece(mov.to(), from_piece);
                self.insert_piece(mov.to(), piece);
            }
            MoveFlags::Quiet | MoveFlags::Capture => {}
            _ => unreachable!("{:?}", mov.flags()),
        }
        self.increment_ply();
        unmake
    }
    pub fn unmake_move(&mut self, unmake: Unmake) {
        self.decrement_ply();
        let mov = unmake.mov;

        match mov.flags() {
            MoveFlags::EnPassant => {
                let back = mov.to().add_rank(-self.active_side.forward()).unwrap();
                self[back] = unmake.captured_piece;
            }
            MoveFlags::QueenCastle if self.white_to_play() => self.swap(Square::A1, Square::D1),
            MoveFlags::QueenCastle => self.swap(Square::A8, Square::D8),
            MoveFlags::KingCastle if self.white_to_play() => self.swap(Square::F1, Square::H1),
            MoveFlags::KingCastle => self.swap(Square::F8, Square::H8),
            MoveFlags::KnightPromotion
            | MoveFlags::KnightPromotionCapture
            | MoveFlags::BishopPromotion
            | MoveFlags::BishopPromotionCapture
            | MoveFlags::RookPromotion
            | MoveFlags::RookPromotionCapture
            | MoveFlags::QueenPromotion
            | MoveFlags::QueenPromotionCapture => {
                self[mov.to()] = Some(self.active_side + Pawn);
            }
            _ => {}
        }

        self[mov.from()] = self[mov.to()];
        if mov.flags() == MoveFlags::EnPassant {
            self[mov.to()] = None;
        } else {
            self[mov.to()] = unmake.captured_piece;
        }

        self.cached = unmake.cached;
        self.en_passant_target_square = unmake.en_passant_target_square;
        self.can_castle = unmake.can_castle;
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
    pub fn get_king_square(&self, side: Side) -> Square {
        self.get(side + King).bitscan()
    }
    #[must_use]
    #[inline]
    pub fn active_king(&self) -> Square {
        self.get_king_square(self.active_side)
    }
    #[must_use]
    #[inline]
    pub fn inactive_king(&self) -> Square {
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
        if let Some(piece) = self[lhs] {
            self.zobrist.xor_piece(lhs, piece);
            self.zobrist.xor_piece(rhs, piece);
            self.remove_piece(lhs, piece);
            self.insert_piece(rhs, piece);
        }
        if let Some(piece) = self[rhs] {
            self.zobrist.xor_piece(lhs, piece);
            self.zobrist.xor_piece(rhs, piece);
            self.remove_piece(rhs, piece);
            self.insert_piece(lhs, piece);
        }
        self.pieces.swap(lhs.0 as usize, rhs.0 as usize);
    }
    #[inline]
    pub fn piece_squares(&self) -> impl Iterator<Item = (Square, Piece)> {
        let pieces = self.pieces;
        Square::all().filter_map(move |sq| pieces[sq].map(|piece| (sq, piece)))
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
}

impl Index<Square> for Board {
    type Output = Option<Piece>;
    fn index(&self, index: Square) -> &Self::Output {
        &self.pieces[index]
    }
}

impl IndexMut<Square> for Board {
    fn index_mut(&mut self, index: Square) -> &mut Self::Output {
        &mut self.pieces[index]
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

// This is bad practice but shame.
impl Deref for Board {
    type Target = Cached;
    fn deref(&self) -> &Self::Target {
        &self.cached
    }
}

impl DerefMut for Board {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cached
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
