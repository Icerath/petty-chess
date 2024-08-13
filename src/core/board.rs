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
    pub active_king_sq: Square,
    pub inactive_king_sq: Square,
    pub zobrist: Zobrist,
    pub piece_bitboards: [Bitboard; 12],
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
            self.piece_bitboards[piece].insert(sq);
            self.zobrist.xor_piece(sq, piece);
            let PieceKind::King = piece.kind() else { continue };
            if piece.side() == self.active_side {
                self.active_king_sq = sq;
            } else {
                self.inactive_king_sq = sq;
            }
        }
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
            self.active_king_sq = mov.to();
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
            self.piece_bitboards[piece].remove(mov.to());
        }
        self.piece_bitboards[from_piece].remove(mov.from());
        self.piece_bitboards[from_piece].insert(mov.to());

        self[mov.from()] = None;
        self[mov.to()] = Some(from_piece);

        match mov.flags() {
            MoveFlags::EnPassant => {
                let back = mov.to().add_rank(-self.active_side.forward()).unwrap();
                let pawn = self[back].unwrap();
                unmake.captured_piece = Some(pawn);
                self.cached.zobrist.xor_piece(back, pawn);
                self.piece_bitboards[pawn].remove(back);
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
                self.piece_bitboards[from_piece].remove(mov.to());
                self.piece_bitboards[piece].insert(mov.to());
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
        std::mem::swap(&mut self.cached.active_king_sq, &mut self.cached.inactive_king_sq);
        self.active_side = !self.active_side;
    }
    #[inline]
    pub fn decrement_ply(&mut self) {
        self.active_side = !self.active_side;
        std::mem::swap(&mut self.cached.active_king_sq, &mut self.cached.inactive_king_sq);
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
    pub fn side_bitboards(&self, side: Side) -> [Bitboard; 6] {
        let offset = if side.is_black() { 0 } else { 6 };
        self.cached.piece_bitboards[offset..offset + 6].try_into().unwrap()
    }
    #[must_use]
    #[inline]
    pub fn side_pieces(&self, side: Side) -> Bitboard {
        self.side_bitboards(side).into_iter().fold(Bitboard(0), |acc, x| acc | x)
    }
    #[must_use]
    #[inline]
    pub fn friendly_bitboards(&self) -> [Bitboard; 6] {
        self.side_bitboards(self.active_side)
    }
    #[must_use]
    #[inline]
    pub fn friendly_pieces(&self) -> Bitboard {
        self.friendly_bitboards().into_iter().fold(Bitboard(0), |acc, x| acc | x)
    }
    #[must_use]
    #[inline]
    pub fn enemy_bitboards(&self) -> [Bitboard; 6] {
        self.side_bitboards(!self.active_side)
    }
    #[must_use]
    #[inline]
    pub fn enemy_pieces(&self) -> Bitboard {
        self.enemy_bitboards().into_iter().fold(Bitboard(0), |acc, x| acc | x)
    }
    #[must_use]
    #[inline]
    pub fn all_pieces(&self) -> Bitboard {
        self.cached.piece_bitboards.into_iter().fold(Bitboard(0), |acc, x| acc | x)
    }
    #[must_use]
    #[inline]
    pub fn get_king_square(&self, side: Side) -> Square {
        if self.active_side == side {
            self.active_king_sq
        } else {
            self.inactive_king_sq
        }
    }
}

impl Board {
    #[inline]
    pub fn swap(&mut self, lhs: Square, rhs: Square) {
        if let Some(piece) = self[lhs] {
            self.zobrist.xor_piece(lhs, piece);
            self.zobrist.xor_piece(rhs, piece);
            self.piece_bitboards[piece].remove(lhs);
            self.piece_bitboards[piece].insert(rhs);
        }
        if let Some(piece) = self[rhs] {
            self.zobrist.xor_piece(lhs, piece);
            self.zobrist.xor_piece(rhs, piece);
            self.piece_bitboards[piece].remove(rhs);
            self.piece_bitboards[piece].insert(lhs);
        }
        self.pieces.swap(lhs.0 as usize, rhs.0 as usize);
    }
    #[inline]
    pub fn pieces(&self) -> impl Iterator<Item = Piece> + '_ {
        Square::all().filter_map(|sq| self[sq])
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
        self.side_pieces(side).contains(sq)
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
