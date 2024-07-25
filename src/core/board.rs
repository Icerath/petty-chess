use core::fmt;
use std::ops::{Index, IndexMut};

use crate::prelude::*;

#[derive(Clone)]
pub struct Board {
    pub pieces: [Option<Piece>; 64],
    pub active_colour: Colour,
    pub can_castle: CanCastle,
    pub en_passant_target_square: Option<Pos>,
    pub halfmove_clock: Option<u8>,
    pub fullmove_counter: Option<u16>,
    pub white_king_pos: Pos,
    pub black_king_pos: Pos,
}

pub struct Unmake {
    board: Board,
    // mov: Move,
    // captured_piece: Option<Piece>,
    // can_castle: CanCastle,
    // en_passant_target_square: Option<Pos>,
}

impl Board {
    /// # Panics
    ///  - TODO
    pub fn make_move(&mut self, mov: Move) -> Unmake {
        let from_piece = self[mov.from()].unwrap();
        let unmake = Unmake {
            board: self.clone(),
            // mov,
            // captured_piece: self[mov.to()],
            // can_castle: self.can_castle,
            // en_passant_target_square: self.en_passant_target_square,
        };
        self.en_passant_target_square = None;
        if from_piece.kind() == PieceKind::King {
            self.set_active_king_pos(mov.to());
            if self.active_colour.is_white() {
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
                debug_assert_eq!(self[back], Some(Piece::new(Pawn, !self.active_colour)));
                self[back] = None;
            }
            MoveFlags::QueenCastle if self.active_colour.is_white() => self.swap(Pos::A1, Pos::D1),
            MoveFlags::QueenCastle => self.swap(Pos::A8, Pos::D8),
            MoveFlags::KingCastle if self.active_colour.is_white() => self.swap(Pos::F1, Pos::H1),
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

        self.active_colour = !self.active_colour;
        unmake
    }
    pub fn unmake_move(&mut self, unmake: Unmake) {
        *self = unmake.board;
        // let mov = unmake.mov;

        // let from_piece = self[mov.to()].unwrap();

        // self.active_colour = !self.active_colour;
        // self.en_passant_target_square = unmake.en_passant_target_square;
        // self.can_castle = unmake.can_castle;

        // match mov.flags() {
        //     MoveFlags::EnPassant => {
        //         let back = mov.to().add_rank(-self.active_colour.forward()).unwrap();
        //         debug_assert!(unmake.captured_piece.is_some());
        //         self[back] = unmake.captured_piece;
        //     }
        //     MoveFlags::KingCastle if self.active_colour.is_white() => self.swap(Pos::F1, Pos::H1),
        //     MoveFlags::KingCastle => self.swap(Pos::F8, Pos::H8),
        //     MoveFlags::QueenCastle if self.active_colour.is_white() => self.swap(Pos::A1, Pos::D1),
        //     MoveFlags::QueenCastle => self.swap(Pos::A8, Pos::D8),
        //     MoveFlags::KnightPromotion
        //     | MoveFlags::KnightPromotionCapture
        //     | MoveFlags::BishopPromotion
        //     | MoveFlags::BishopPromotionCapture
        //     | MoveFlags::RookPromotion
        //     | MoveFlags::RookPromotionCapture
        //     | MoveFlags::QueenPromotion
        //     | MoveFlags::QueenPromotionCapture => {
        //         self[mov.to()] = Some(Piece::new(Pawn, self.active_colour))
        //     }
        //     _ => {}
        // }

        // self[mov.from()] = Some(from_piece);
        // if mov.flags() != MoveFlags::EnPassant {
        //     self[mov.to()] = unmake.captured_piece;
        // }
    }

    #[inline]
    pub fn swap(&mut self, lhs: Pos, rhs: Pos) {
        self.pieces.swap(lhs.0 as usize, rhs.0 as usize);
    }
    #[must_use]
    #[inline]
    pub fn active_king_pos(&self) -> Pos {
        match self.active_colour {
            Colour::White => self.white_king_pos,
            Colour::Black => self.black_king_pos,
        }
    }
    #[must_use]
    #[inline]
    pub fn inactive_king_pos(&self) -> Pos {
        match self.active_colour {
            Colour::White => self.black_king_pos,
            Colour::Black => self.white_king_pos,
        }
    }
    #[inline]
    fn set_active_king_pos(&mut self, pos: Pos) {
        match self.active_colour {
            Colour::White => self.white_king_pos = pos,
            Colour::Black => self.black_king_pos = pos,
        }
    }
    pub fn find_king_positions(&mut self) {
        for pos in (0..64).map(Pos) {
            let Some(piece) = self[pos] else { continue };
            if piece == Piece::new(King, White) {
                self.white_king_pos = pos;
            }
            if piece == Piece::new(King, Black) {
                self.black_king_pos = pos;
            }
        }
    }
    #[must_use]
    pub fn gen_pseudolegal_moves(&self) -> Moves {
        MoveGenerator::new_pseudo_legal(self.clone()).gen_moves()
    }
    #[must_use]
    pub fn gen_legal_moves(&self) -> Moves {
        MoveGenerator::new(self.clone()).gen_moves()
    }
    #[must_use]
    pub fn gen_capture_moves(&self) -> Moves {
        let mut moves = self.gen_legal_moves();
        moves.retain(|mov| mov.flags().is_capture());
        moves
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
