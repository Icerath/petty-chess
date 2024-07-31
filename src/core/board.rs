use core::fmt;
use std::{
    hash::Hash,
    ops::{Deref, DerefMut, Index, IndexMut},
};

use rustc_hash::FxHashMap;

use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct Pieces(pub [Option<Piece>; 64]);

impl PartialEq for Pieces {
    fn eq(&self, other: &Self) -> bool {
        let lhs = unsafe { std::mem::transmute::<[Option<Piece>; 64], [u8; 64]>(self.0) };
        let rhs = unsafe { std::mem::transmute::<[Option<Piece>; 64], [u8; 64]>(other.0) };
        lhs.eq(&rhs)
    }
}

impl Eq for Pieces {}

impl Hash for Pieces {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { std::mem::transmute::<[Option<Piece>; 64], [u8; 64]>(self.0) }.hash(state);
    }
}

pub struct Board {
    pub pieces: [Option<Piece>; 64],
    pub active_colour: Colour,
    pub can_castle: CanCastle,
    pub en_passant_target_square: Option<Pos>,
    pub halfmove_clock: u8,
    pub fullmove_counter: u16,
    pub cached: Cached,
    pub seen_positions: FxHashMap<(Pieces, Colour), u32>,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Cached {
    pub active_king_pos: Pos,
    pub inactive_king_pos: Pos,

    pub piece_bitboards: [[Bitboard; 6]; 2],
}

pub struct Unmake {
    cached: Cached,
    mov: Move,
    captured_piece: Option<Piece>,
    can_castle: CanCastle,
    en_passant_target_square: Option<Pos>,
}

impl Board {
    /// Returns whether the current position has been seen before
    #[inline]
    #[must_use]
    pub fn seen_position(&self) -> u32 {
        self.seen_positions.get(&(Pieces(self.pieces), self.active_colour)).copied().unwrap_or(0)
    }
    pub fn create_cache(&mut self) {
        for (pos, piece) in self.piece_positions() {
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
        self.en_passant_target_square = None;
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

        self[mov.from()] = None;
        self[mov.to()] = Some(from_piece);

        match mov.flags() {
            MoveFlags::EnPassant => {
                let back = mov.to().add_rank(-self.active_colour.forward()).unwrap();
                debug_assert_eq!(self[back], Some(!self.active_colour + Pawn));
                unmake.captured_piece = self[back];
                self[back] = None;
            }
            MoveFlags::QueenCastle if self.white_to_play() => self.swap(Pos::A1, Pos::D1),
            MoveFlags::QueenCastle => self.swap(Pos::A8, Pos::D8),
            MoveFlags::KingCastle if self.white_to_play() => self.swap(Pos::F1, Pos::H1),
            MoveFlags::KingCastle => self.swap(Pos::F8, Pos::H8),
            MoveFlags::DoublePawnPush => {
                let back = mov.to().add_rank(-self.active_colour.forward()).unwrap();
                self.en_passant_target_square = Some(back);
            }
            MoveFlags::KnightPromotion | MoveFlags::KnightPromotionCapture => {
                self[mov.to()] = Some(from_piece.with_kind(PieceKind::Knight));
            }
            MoveFlags::BishopPromotion | MoveFlags::BishopPromotionCapture => {
                self[mov.to()] = Some(from_piece.with_kind(PieceKind::Bishop));
            }
            MoveFlags::RookPromotion | MoveFlags::RookPromotionCapture => {
                self[mov.to()] = Some(from_piece.with_kind(PieceKind::Rook));
            }
            MoveFlags::QueenPromotion | MoveFlags::QueenPromotionCapture => {
                self[mov.to()] = Some(from_piece.with_kind(PieceKind::Queen));
            }
            _ => {}
        }
        self.increment_ply();
        unmake
    }
    pub fn unmake_move(&mut self, unmake: Unmake) {
        self.decrement_ply();
        self.cached = unmake.cached;
        self.en_passant_target_square = unmake.en_passant_target_square;
        self.can_castle = unmake.can_castle;

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
    }
    #[inline]
    pub fn increment_ply(&mut self) {
        if self.black_to_play() {
            self.fullmove_counter += 1;
        }
        self.halfmove_clock += 1;
        std::mem::swap(&mut self.cached.active_king_pos, &mut self.cached.inactive_king_pos);
        self.active_colour = !self.active_colour;
        *self.seen_positions.entry((Pieces(self.pieces), self.active_colour)).or_default() += 1;
    }
    #[inline]
    pub fn decrement_ply(&mut self) {
        *self.seen_positions.get_mut(&(Pieces(self.pieces), self.active_colour)).unwrap() -= 1;
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
}

impl Board {
    #[inline]
    pub fn swap(&mut self, lhs: Pos, rhs: Pos) {
        self.pieces.swap(lhs.0 as usize, rhs.0 as usize);
    }
    #[inline]
    pub fn pieces(&self) -> impl Iterator<Item = Piece> + '_ {
        (0..64).map(Pos).filter_map(|pos| self[pos])
    }
    #[inline]
    pub fn piece_positions(&self) -> impl Iterator<Item = (Pos, Piece)> {
        let pieces = self.pieces;
        (0..64).map(Pos).filter_map(move |pos| pieces[pos.0 as usize].map(|piece| (pos, piece)))
    }
}

impl Index<Pos> for Board {
    type Output = Option<Piece>;
    fn index(&self, index: Pos) -> &Self::Output {
        &self.pieces[index.0 as usize]
    }
}

impl IndexMut<Pos> for Board {
    fn index_mut(&mut self, index: Pos) -> &mut Self::Output {
        &mut self.pieces[index.0 as usize]
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
