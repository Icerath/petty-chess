use std::marker::PhantomData;

use super::magic::Magic;
use crate::prelude::*;
pub const DIRECTION_OFFSETS: [i8; 8] = [8, -8, -1, 1, 7, -7, 9, -9];
pub const NUM_SQUARES_TO_EDGE: [[i8; 8]; 64] = compute_num_squares_to_edge();
const KING_MOVES: [Bitboard; 64] = compute_king_moves();
const KNIGHT_MOVES: [Bitboard; 64] = compute_knight_moves();
const ATTACK_PAWN_MOVES: [[Bitboard; 64]; 2] = compute_pawn_moves();

pub struct CapturesOnly;
pub struct FullGen;

pub trait GenType {
    const CAPTURES_ONLY: bool;
}

impl GenType for CapturesOnly {
    const CAPTURES_ONLY: bool = true;
}

impl GenType for FullGen {
    const CAPTURES_ONLY: bool = false;
}

pub struct MoveGenerator<'a, G: GenType = FullGen> {
    moves: Moves,
    board: &'a mut Board,
    pub queen_knight_promote_only: bool,
    magic: &'static Magic,
    ty: PhantomData<G>,
}

impl Board {
    #[must_use]
    pub fn gen_pseudolegal_moves(&mut self) -> Moves {
        MoveGenerator::<FullGen>::new(self).gen_pseudolegal_moves()
    }
    #[must_use]
    pub fn gen_legal_moves(&mut self) -> Moves {
        let mut movegen = MoveGenerator::<FullGen>::new(self);
        movegen.queen_knight_promote_only = false;
        movegen.gen_legal_moves()
    }
    #[must_use]
    pub fn gen_capture_moves(&mut self) -> Moves {
        MoveGenerator::<CapturesOnly>::new(self).gen_legal_moves()
    }
    #[must_use]
    pub fn gen_pseudolegal_capture_moves(&mut self) -> Moves {
        MoveGenerator::<CapturesOnly>::new(self).gen_pseudolegal_moves()
    }
}

impl<'a, G: GenType> MoveGenerator<'a, G> {
    #[must_use]
    pub fn new(board: &'a mut Board) -> Self {
        Self {
            moves: Moves::default(),
            board,
            queen_knight_promote_only: true,
            magic: Magic::get(),
            ty: PhantomData,
        }
    }
    #[must_use]
    pub fn gen_legal_moves(&mut self) -> Moves {
        let mut moves = self.gen_pseudolegal_moves();
        moves.retain(|&mut mov| self.is_legal(mov));
        moves
    }
    #[must_use]
    pub fn gen_pseudolegal_moves(&mut self) -> Moves {
        let pieces = self.board.friendly_bitboards();
        let all_pieces = self.board.all_pieces();

        pieces[Pawn].for_each(|from| self.gen_pawn_moves(from));
        pieces[Knight].for_each(|from| self.push_squares(from, KNIGHT_MOVES[from]));
        pieces[Bishop].for_each(|from| self.push_squares(from, self.magic.bishop_attacks(from, all_pieces)));
        pieces[Rook].for_each(|from| self.push_squares(from, self.magic.rook_attacks(from, all_pieces)));
        pieces[Queen].for_each(|from| self.push_squares(from, self.magic.queen_attacks(from, all_pieces)));
        if let Some(king_pos) = self.board.active_king() {
            self.gen_king_moves(king_pos);
        }
        std::mem::take(&mut self.moves)
    }
    #[must_use]
    #[inline]
    pub fn is_legal(&mut self, mov: Move) -> bool {
        let Some(king_pos) = self.board.active_king() else { return true };

        if mov.flags() == MoveFlags::KingCastle || mov.flags() == MoveFlags::QueenCastle {
            let map = self.attack_map();
            let squares = match (self.board.active_side, mov.flags() == MoveFlags::KingCastle) {
                (Side::White, true) => [Square::F1, Square::G1],
                (Side::White, false) => [Square::C1, Square::D1],
                (Side::Black, true) => [Square::F8, Square::G8],
                (Side::Black, false) => [Square::C8, Square::D8],
            };
            if map.contains(squares[0]) || map.contains(squares[1]) || map.contains(king_pos) {
                return false;
            }
        }
        let unmake = self.board.make_move(mov);
        let is_attacked = self.board.inactive_king().is_some_and(|pos| self.is_square_attacked(pos));
        self.board.unmake_move(unmake);
        !is_attacked
    }
    #[must_use]
    #[inline]
    pub fn attack_map(&mut self) -> Bitboard {
        self.gen_attack_map()
    }
    // Generate attack map for enemy pieces
    #[inline]
    fn gen_attack_map(&self) -> Bitboard {
        let mut attacked_squares = Bitboard(0);
        let side = !self.board.active_side;
        let enemy_pieces = self.board.enemy_bitboards();
        let all_pieces = self.board.all_pieces();

        enemy_pieces[Pawn].for_each(|from| attacked_squares |= ATTACK_PAWN_MOVES[side as usize][from]);
        enemy_pieces[Knight].for_each(|from| attacked_squares |= KNIGHT_MOVES[from]);
        enemy_pieces[Bishop].for_each(|from| attacked_squares |= self.magic.bishop_attacks(from, all_pieces));
        enemy_pieces[Rook].for_each(|from| attacked_squares |= self.magic.rook_attacks(from, all_pieces));
        enemy_pieces[Queen].for_each(|from| attacked_squares |= self.magic.queen_attacks(from, all_pieces));
        if let Some(king) = self.board.inactive_king() {
            attacked_squares |= KING_MOVES[king];
        }
        attacked_squares
    }
    #[inline]
    pub(crate) fn pawn_attack_map(&self) -> Bitboard {
        let mut attacked_squares = Bitboard(0);
        let side = !self.board.active_side;
        self.board.get(side + Pawn).for_each(|from| attacked_squares |= ATTACK_PAWN_MOVES[side as usize][from]);
        attacked_squares
    }
    #[inline]
    fn push_squares(&mut self, from: Square, mut squares: Bitboard) {
        squares &= !self.board[self.board.active_side];
        squares.for_each(|sq| {
            if self.board.is_piece_at(sq) {
                self.moves.push(Move::new(from, sq, MoveFlags::Capture));
            } else if !G::CAPTURES_ONLY {
                self.moves.push(Move::new(from, sq, MoveFlags::Quiet));
            }
        });
    }
    fn gen_pawn_moves(&mut self, from: Square) {
        let forward = self.board.active_side.forward();

        let can_promote = (self.board.white_to_play() && from.rank().0 == 6)
            || (self.board.black_to_play() && from.rank().0 == 1);

        if !G::CAPTURES_ONLY {
            let to = Square::try_from(i8::from(from) + forward * 8).unwrap();
            if !self.board.is_piece_at(to) {
                let can_double_push = (self.board.white_to_play() && from.rank().0 == 1)
                    || (self.board.black_to_play() && from.rank().0 == 6);

                if !can_promote {
                    self.moves.push(Move::new(from, to, MoveFlags::Quiet));
                }

                if can_double_push {
                    let to = Square::try_from(i8::from(from) + forward * 16).unwrap();
                    if !self.board.is_piece_at(to) {
                        self.moves.push(Move::new(from, to, MoveFlags::DoublePawnPush));
                    }
                } else if can_promote {
                    self.moves.push(Move::new(from, to, MoveFlags::QueenPromotion));
                    self.moves.push(Move::new(from, to, MoveFlags::KnightPromotion));
                    if !self.queen_knight_promote_only {
                        self.moves.push(Move::new(from, to, MoveFlags::BishopPromotion));
                        self.moves.push(Move::new(from, to, MoveFlags::RookPromotion));
                    }
                }
            }
        }
        if let Some(to) = Square::try_from(i8::from(from) + forward * 8).unwrap().add_file(1) {
            if self.board.is_side(to, !self.board.active_side) {
                if can_promote {
                    self.moves.push(Move::new(from, to, MoveFlags::QueenPromotionCapture));
                    self.moves.push(Move::new(from, to, MoveFlags::KnightPromotionCapture));
                    if !self.queen_knight_promote_only {
                        self.moves.push(Move::new(from, to, MoveFlags::BishopPromotionCapture));
                        self.moves.push(Move::new(from, to, MoveFlags::RookPromotionCapture));
                    }
                } else {
                    self.moves.push(Move::new(from, to, MoveFlags::Capture));
                }
            }
        }
        if let Some(to) = Square::try_from(i8::from(from) + forward * 8).unwrap().add_file(-1) {
            if self.board.is_side(to, !self.board.active_side) {
                if can_promote {
                    self.moves.push(Move::new(from, to, MoveFlags::KnightPromotionCapture));
                    self.moves.push(Move::new(from, to, MoveFlags::QueenPromotionCapture));
                    if !self.queen_knight_promote_only {
                        self.moves.push(Move::new(from, to, MoveFlags::BishopPromotionCapture));
                        self.moves.push(Move::new(from, to, MoveFlags::RookPromotionCapture));
                    }
                } else {
                    self.moves.push(Move::new(from, to, MoveFlags::Capture));
                }
            }
        }
        if let Some(en_passant) = self.board.en_passant_target_square {
            if ((en_passant.file().0 as i8 - from.file().0 as i8).abs()) <= 1
                && from.rank().0 as i8 == (en_passant.rank().0 as i8 - forward)
            {
                self.moves.push(Move::new(from, en_passant, MoveFlags::EnPassant));
            }
        }
    }
    fn gen_king_moves(&mut self, from: Square) {
        self.push_squares(from, KING_MOVES[from]);
        if G::CAPTURES_ONLY {
            return;
        }
        if self.board.white_to_play() {
            if self.board.can_castle.contains(CanCastle::WHITE_KING_SIDE)
                && !self.board.is_piece_at(Square::F1)
                && !self.board.is_piece_at(Square::G1)
            {
                self.moves.push(Move::new(from, Square::G1, MoveFlags::KingCastle));
            }
            if self.board.can_castle.contains(CanCastle::WHITE_QUEEN_SIDE)
                && !self.board.is_piece_at(Square::C1)
                && !self.board.is_piece_at(Square::D1)
                && !self.board.is_piece_at(Square::B1)
            {
                self.moves.push(Move::new(from, Square::C1, MoveFlags::QueenCastle));
            }
        } else {
            if self.board.can_castle.contains(CanCastle::BLACK_KING_SIDE)
                && !self.board.is_piece_at(Square::F8)
                && !self.board.is_piece_at(Square::G8)
            {
                self.moves.push(Move::new(from, Square::G8, MoveFlags::KingCastle));
            }
            if self.board.can_castle.contains(CanCastle::BLACK_QUEEN_SIDE)
                && !self.board.is_piece_at(Square::B8)
                && !self.board.is_piece_at(Square::C8)
                && !self.board.is_piece_at(Square::D8)
            {
                self.moves.push(Move::new(from, Square::C8, MoveFlags::QueenCastle));
            }
        }
    }
    #[inline]
    pub fn is_square_attacked(&mut self, sq: Square) -> bool {
        self.board.active_side = !self.board.active_side;
        let atk_map = self.gen_attack_map();
        self.board.active_side = !self.board.active_side;
        atk_map.contains(sq)
    }
}

const fn compute_pawn_moves() -> [[Bitboard; 64]; 2] {
    let mut black_squares = [Bitboard(0); 64];
    let mut white_squares = [Bitboard(0); 64];

    let mut index = 0;
    while index < 64 {
        let sq = unsafe { Square::new_int_unchecked(index as u8) };

        let num_up = sq.rank().0 as i8;
        let num_down = 7 - sq.rank().0 as i8;
        let num_left = sq.file().0 as i8;
        let num_right = 7 - sq.file().0 as i8;

        let up = -8;
        let down = -up;
        let left = -1;
        let right = -left;

        let mut bitboard = Bitboard(0);
        if num_up > 0 {
            bitboard.0 |= if num_left > 0 { 1 << (index + up + left) } else { 0 };
            bitboard.0 |= if num_right > 0 { 1 << (index + up + right) } else { 0 };
        }
        white_squares[index as usize] = bitboard;

        let mut bitboard = Bitboard(0);
        if num_down > 0 {
            bitboard.0 |= if num_left > 0 { 1 << (index + down + left) } else { 0 };
            bitboard.0 |= if num_right > 0 { 1 << (index + down + right) } else { 0 };
        }
        black_squares[index as usize] = bitboard;

        index += 1;
    }
    [white_squares, black_squares]
}

const fn compute_knight_moves() -> [Bitboard; 64] {
    let mut squares = [Bitboard(0); 64];

    let mut index = 0;
    while index < 64 {
        let sq = unsafe { Square::new_int_unchecked(index as u8) };

        let num_up = 7 - sq.rank().0 as i8;
        let num_down = sq.rank().0 as i8;
        let num_left = sq.file().0 as i8;
        let num_right = 7 - sq.file().0 as i8;

        let mut bitboard = Bitboard(0);

        let up = 8;
        let down = -up;
        let left = -1;
        let right = -left;

        bitboard.0 |= if num_up >= 2 && num_left >= 1 { 1 << (index + up * 2 + left) } else { 0 };
        bitboard.0 |= if num_up >= 2 && num_right >= 1 { 1 << (index + up * 2 + right) } else { 0 };
        bitboard.0 |= if num_down >= 2 && num_left >= 1 { 1 << (index + down * 2 + left) } else { 0 };
        bitboard.0 |= if num_down >= 2 && num_right >= 1 { 1 << (index + down * 2 + right) } else { 0 };

        bitboard.0 |= if num_left >= 2 && num_up >= 1 { 1 << (index + up + left * 2) } else { 0 };
        bitboard.0 |= if num_right >= 2 && num_up >= 1 { 1 << (index + up + right * 2) } else { 0 };
        bitboard.0 |= if num_left >= 2 && num_down >= 1 { 1 << (index + down + left * 2) } else { 0 };
        bitboard.0 |= if num_right >= 2 && num_down >= 1 { 1 << (index + down + right * 2) } else { 0 };

        squares[index as usize] = bitboard;
        index += 1;
    }
    squares
}

const fn compute_king_moves() -> [Bitboard; 64] {
    let mut squares = [Bitboard(0); 64];

    let mut index = 0;
    while index < 64 {
        let sq = unsafe { Square::new_int_unchecked(index as u8) };

        let num_up = 7 - sq.rank().0;
        let num_down = sq.rank().0;
        let num_left = sq.file().0;
        let num_right = 7 - sq.file().0;

        let mut bitboard = Bitboard(0);

        let up = 8;
        let down = -up;
        let left = -1;
        let right = -left;

        bitboard.0 |= if num_up > 0 { 1 << (index + up) } else { 0 };
        bitboard.0 |= if num_down > 0 { 1 << (index + down) } else { 0 };
        bitboard.0 |= if num_left > 0 { 1 << (index + left) } else { 0 };
        bitboard.0 |= if num_right > 0 { 1 << (index + right) } else { 0 };

        bitboard.0 |= if num_up > 0 && num_left > 0 { 1 << (index + up + left) } else { 0 };
        bitboard.0 |= if num_up > 0 && num_right > 0 { 1 << (index + up + right) } else { 0 };
        bitboard.0 |= if num_down > 0 && num_left > 0 { 1 << (index + down + left) } else { 0 };
        bitboard.0 |= if num_down > 0 && num_right > 0 { 1 << (index + down + right) } else { 0 };

        squares[index as usize] = bitboard;
        index += 1;
    }
    squares
}

const fn compute_num_squares_to_edge() -> [[i8; 8]; 64] {
    const fn min(lhs: i8, rhs: i8) -> i8 {
        if lhs < rhs {
            lhs
        } else {
            rhs
        }
    }

    let mut squares = [[0; 8]; 64];

    let mut index = 0;
    while index < 64 {
        let sq = unsafe { Square::new_int_unchecked(index as u8) };

        let num_up = 7 - sq.rank().0 as i8;
        let num_down = sq.rank().0 as i8;
        let num_left = sq.file().0 as i8;
        let num_right = 7 - sq.file().0 as i8;

        squares[sq.int() as usize] = [
            num_up,
            num_down,
            num_left,
            num_right,
            min(num_up, num_left),
            min(num_down, num_right),
            min(num_up, num_right),
            min(num_down, num_left),
        ];
        index += 1;
    }
    squares
}
