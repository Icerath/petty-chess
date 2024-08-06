use core::fmt;
use std::ops::{Deref, DerefMut, Index, IndexMut};

use crate::prelude::*;

#[derive(Clone)]
pub struct Board {
    pub pieces: [Option<Piece>; 64],
    pub active_colour: Colour,
    pub can_castle: CanCastle,
    pub en_passant_target_square: Option<Pos>,
    pub halfmove_clock: u8,
    pub fullmove_counter: u16,
    pub cached: Cached,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Cached {
    pub active_king_pos: Pos,
    pub inactive_king_pos: Pos,
    pub zobrist: Zobrist,
    pub piece_bitboards: [Bitboard; 12],
}

pub struct Unmake {
    cached: Cached,
    mov: Move,
    captured_piece: Option<Piece>,
    can_castle: CanCastle,
    en_passant_target_square: Option<Pos>,
}

impl Board {
    pub fn create_cache(&mut self) {
        self.zobrist = Zobrist::default();
        if self.black_to_play() {
            self.zobrist.xor_side_to_move();
        }
        self.cached.zobrist.xor_can_castle(self.can_castle);
        if let Some(square) = self.en_passant_target_square {
            self.cached.zobrist.xor_en_passant(square);
        }
        for (pos, piece) in self.piece_positions() {
            self.piece_bitboards[piece].insert(pos);
            self.zobrist.xor_piece(pos, piece);
            let PieceKind::King = piece.kind() else { continue };
            if piece.colour() == self.active_colour {
                self.active_king_pos = pos;
            } else {
                self.inactive_king_pos = pos;
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
        if let Some(square) = self.en_passant_target_square {
            self.zobrist.xor_en_passant(square);
        }
        self.en_passant_target_square = None;
        self.cached.zobrist.xor_can_castle(self.can_castle);
        if from_piece.kind() == PieceKind::King {
            self.active_king_pos = mov.to();
            if self.white_to_play() {
                self.can_castle.remove(CanCastle::BOTH_WHITE);
            } else {
                self.can_castle.remove(CanCastle::BOTH_BLACK);
            }
        }
        for pos in [mov.from(), mov.to()] {
            match pos {
                Pos::A1 => self.can_castle.remove(CanCastle::WHITE_QUEEN_SIDE),
                Pos::H1 => self.can_castle.remove(CanCastle::WHITE_KING_SIDE),
                Pos::A8 => self.can_castle.remove(CanCastle::BLACK_QUEEN_SIDE),
                Pos::H8 => self.can_castle.remove(CanCastle::BLACK_KING_SIDE),
                _ => {}
            }
        }

        self.cached.zobrist.xor_can_castle(self.can_castle);
        self.zobrist.xor_side_to_move();
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
                let back = mov.to().add_rank(-self.active_colour.forward()).unwrap();
                let pawn = self[back].unwrap();
                unmake.captured_piece = Some(pawn);
                self.cached.zobrist.xor_piece(back, pawn);
                self.piece_bitboards[pawn].remove(back);
                self[back] = None;
            }
            MoveFlags::QueenCastle if self.white_to_play() => {
                self.swap(Pos::A1, Pos::D1);
            }
            MoveFlags::QueenCastle => {
                self.swap(Pos::A8, Pos::D8);
            }
            MoveFlags::KingCastle if self.white_to_play() => {
                self.swap(Pos::F1, Pos::H1);
            }
            MoveFlags::KingCastle => {
                self.swap(Pos::F8, Pos::H8);
            }
            MoveFlags::DoublePawnPush => {
                let back = mov.to().add_rank(-self.active_colour.forward()).unwrap();
                self.en_passant_target_square = Some(back);
                self.zobrist.xor_en_passant(back);
            }
            flags if flags.promotion().is_some() => {
                let piece = self.active_colour + PieceKind::from(flags.promotion().unwrap());
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
                let back = mov.to().add_rank(-self.active_colour.forward()).unwrap();
                self[back] = unmake.captured_piece;
            }
            MoveFlags::QueenCastle if self.white_to_play() => self.swap(Pos::A1, Pos::D1),
            MoveFlags::QueenCastle => self.swap(Pos::A8, Pos::D8),
            MoveFlags::KingCastle if self.white_to_play() => self.swap(Pos::F1, Pos::H1),
            MoveFlags::KingCastle => self.swap(Pos::F8, Pos::H8),
            MoveFlags::KnightPromotion
            | MoveFlags::KnightPromotionCapture
            | MoveFlags::BishopPromotion
            | MoveFlags::BishopPromotionCapture
            | MoveFlags::RookPromotion
            | MoveFlags::RookPromotionCapture
            | MoveFlags::QueenPromotion
            | MoveFlags::QueenPromotionCapture => {
                self[mov.to()] = Some(self.active_colour + Pawn);
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
    #[inline]
    pub fn increment_ply(&mut self) {
        if self.black_to_play() {
            self.fullmove_counter += 1;
        }
        self.halfmove_clock += 1;
        std::mem::swap(&mut self.cached.active_king_pos, &mut self.cached.inactive_king_pos);
        self.active_colour = !self.active_colour;
    }
    #[inline]
    pub fn decrement_ply(&mut self) {
        self.active_colour = !self.active_colour;
        std::mem::swap(&mut self.cached.active_king_pos, &mut self.cached.inactive_king_pos);
        self.halfmove_clock -= 1;
        if self.black_to_play() {
            self.fullmove_counter -= 1;
        }
    }
    #[inline]
    #[must_use]
    pub fn white_to_play(&self) -> bool {
        self.active_colour.is_white()
    }
    #[inline]
    #[must_use]
    pub fn black_to_play(&self) -> bool {
        self.active_colour.is_black()
    }
    #[must_use]
    #[inline]
    pub fn friendly_pieces(&self) -> Bitboard {
        let offset = if self.active_colour.is_black() { 0 } else { 6 };
        self.cached.piece_bitboards[offset..offset + 6].iter().fold(Bitboard(0), |acc, &x| acc | x)
    }
    #[must_use]
    #[inline]
    pub fn enemy_pieces(&self) -> Bitboard {
        let offset = if self.active_colour.is_white() { 0 } else { 6 };
        self.cached.piece_bitboards[offset..offset + 6].iter().fold(Bitboard(0), |acc, &x| acc | x)
    }
    #[must_use]
    #[inline]
    pub fn all_pieces(&self) -> Bitboard {
        self.cached.piece_bitboards.iter().fold(Bitboard(0), |acc, &x| acc | x)
    }
}

impl Board {
    #[inline]
    pub fn swap(&mut self, lhs: Pos, rhs: Pos) {
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
        (0..64).map(Pos).filter_map(|pos| self[pos])
    }
    #[inline]
    pub fn piece_positions(&self) -> impl Iterator<Item = (Pos, Piece)> {
        let pieces = self.pieces;
        (0..64).map(Pos).filter_map(move |pos| pieces[pos].map(|piece| (pos, piece)))
    }
}

impl Index<Pos> for Board {
    type Output = Option<Piece>;
    fn index(&self, index: Pos) -> &Self::Output {
        &self.pieces[index]
    }
}

impl IndexMut<Pos> for Board {
    fn index_mut(&mut self, index: Pos) -> &mut Self::Output {
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
