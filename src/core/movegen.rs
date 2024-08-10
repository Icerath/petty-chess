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

trait GenType {
    const CAPTURES_ONLY: bool;
}

impl GenType for CapturesOnly {
    const CAPTURES_ONLY: bool = true;
}

impl GenType for FullGen {
    const CAPTURES_ONLY: bool = false;
}

#[allow(private_bounds)]
pub struct MoveGenerator<'a, G: GenType = FullGen> {
    moves: Moves,
    board: &'a mut Board,
    attacked_squares: Option<Bitboard>,
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

#[allow(private_bounds)]
impl<'a, G: GenType> MoveGenerator<'a, G> {
    #[must_use]
    pub fn new(board: &'a mut Board) -> Self {
        Self {
            moves: Moves::default(),
            board,
            attacked_squares: None,
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
    pub fn gen_pseudolegal_moves(&mut self) -> Moves {
        let pieces = self.board.friendly_bitboards();
        let all_pieces = self.board.all_pieces();

        pieces[Pawn].for_each(|from| self.gen_pawn_moves(from));
        pieces[Knight].for_each(|from| self.push_squares(from, KNIGHT_MOVES[from]));
        pieces[Bishop].for_each(|from| self.push_squares(from, self.magic.bishop_attacks(from, all_pieces)));
        pieces[Rook].for_each(|from| self.push_squares(from, self.magic.rook_attacks(from, all_pieces)));
        pieces[Queen].for_each(|from| self.push_squares(from, self.magic.queen_attacks(from, all_pieces)));
        self.gen_king_moves(self.board.active_king_pos);
        std::mem::take(&mut self.moves)
    }
    #[must_use]
    #[inline]
    pub fn is_legal(&mut self, mov: Move) -> bool {
        if mov.flags() == MoveFlags::KingCastle || mov.flags() == MoveFlags::QueenCastle {
            let map = self.attack_map();
            let squares = match (self.board.active_colour, mov.flags() == MoveFlags::KingCastle) {
                (Colour::White, true) => [Square::F1, Square::G1],
                (Colour::White, false) => [Square::C1, Square::D1],
                (Colour::Black, true) => [Square::F8, Square::G8],
                (Colour::Black, false) => [Square::C8, Square::D8],
            };
            if map.contains(squares[0]) || map.contains(squares[1]) || map.contains(self.board.active_king_pos)
            {
                return false;
            }
        }
        let unmake = self.board.make_move(mov);
        let is_attacked = self.is_square_attacked(self.board.inactive_king_pos);
        self.board.unmake_move(unmake);
        !is_attacked
    }
    pub fn attack_map(&mut self) -> Bitboard {
        if let Some(attack_map) = self.attacked_squares {
            return attack_map;
        }
        let attack_map = self.gen_attack_map();
        self.attacked_squares = Some(attack_map);
        attack_map
    }
    // Generate attack map for enemy pieces
    #[allow(clippy::needless_range_loop)]
    #[must_use]
    fn gen_attack_map(&self) -> Bitboard {
        let mut attacked_squares = Bitboard(0);
        let colour = !self.board.active_colour;
        let enemy_pieces = self.board.enemy_bitboards();
        let all_pieces = self.board.all_pieces();

        enemy_pieces[Pawn].for_each(|from| attacked_squares |= ATTACK_PAWN_MOVES[colour as usize][from]);
        enemy_pieces[Knight].for_each(|from| attacked_squares |= KNIGHT_MOVES[from]);
        enemy_pieces[Bishop].for_each(|from| attacked_squares |= self.magic.bishop_attacks(from, all_pieces));
        enemy_pieces[Rook].for_each(|from| attacked_squares |= self.magic.rook_attacks(from, all_pieces));
        enemy_pieces[Queen].for_each(|from| attacked_squares |= self.magic.queen_attacks(from, all_pieces));
        attacked_squares |= KING_MOVES[self.board.inactive_king_pos];
        attacked_squares
    }
    #[inline]
    #[must_use]
    pub fn pawn_attack_map(&self) -> Bitboard {
        let mut attacked_squares = Bitboard(0);
        let colour = !self.board.active_colour;
        self.board[colour + Pawn].for_each(|from| attacked_squares |= ATTACK_PAWN_MOVES[colour as usize][from]);
        attacked_squares
    }
    #[inline]
    fn push_squares(&mut self, from: Square, mut squares: Bitboard) {
        squares &= !self.board.friendly_pieces();
        squares.for_each(|sq| {
            if self.board[sq].is_some() {
                self.moves.push(Move::new(from, sq, MoveFlags::Capture));
            } else if !G::CAPTURES_ONLY {
                self.moves.push(Move::new(from, sq, MoveFlags::Quiet));
            }
        });
    }
    fn gen_pawn_moves(&mut self, from: Square) {
        let forward = self.board.active_colour.forward();

        let can_promote = (self.board.white_to_play() && from.rank().0 == 6)
            || (self.board.black_to_play() && from.rank().0 == 1);

        if !G::CAPTURES_ONLY {
            let to = Square(from.0 + forward * 8);
            if self.board[to].is_none() {
                let can_double_push = (self.board.white_to_play() && from.rank().0 == 1)
                    || (self.board.black_to_play() && from.rank().0 == 6);

                if !can_promote {
                    self.moves.push(Move::new(from, to, MoveFlags::Quiet));
                }

                if can_double_push {
                    let to = Square(from.0 + forward * 16);
                    if self.board[to].is_none() {
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
        if let Some(to) = Square(from.0 + forward * 8).add_file(1) {
            if self.board[to].map(Piece::colour) == Some(!self.board.active_colour) {
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
        if let Some(to) = Square(from.0 + forward * 8).add_file(-1) {
            if self.board[to].map(Piece::colour) == Some(!self.board.active_colour) {
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
            if ((en_passant.file().0 - from.file().0).abs()) <= 1
                && from.rank().0 == (en_passant.rank().0 - forward)
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
                && self.board[Square::F1].is_none()
                && self.board[Square::G1].is_none()
            {
                self.moves.push(Move::new(from, Square::G1, MoveFlags::KingCastle));
            }
            if self.board.can_castle.contains(CanCastle::WHITE_QUEEN_SIDE)
                && self.board[Square::C1].is_none()
                && self.board[Square::D1].is_none()
                && self.board[Square::B1].is_none()
            {
                self.moves.push(Move::new(from, Square::C1, MoveFlags::QueenCastle));
            }
        } else {
            if self.board.can_castle.contains(CanCastle::BLACK_KING_SIDE)
                && self.board[Square::F8].is_none()
                && self.board[Square::G8].is_none()
            {
                self.moves.push(Move::new(from, Square::G8, MoveFlags::KingCastle));
            }
            if self.board.can_castle.contains(CanCastle::BLACK_QUEEN_SIDE)
                && self.board[Square::B8].is_none()
                && self.board[Square::C8].is_none()
                && self.board[Square::D8].is_none()
            {
                self.moves.push(Move::new(from, Square::C8, MoveFlags::QueenCastle));
            }
        }
    }
    #[inline]
    pub fn is_square_attacked(&mut self, sq: Square) -> bool {
        self.board.active_colour = !self.board.active_colour;
        std::mem::swap(&mut self.board.cached.active_king_pos, &mut self.board.cached.inactive_king_pos);
        let atk_map = self.gen_attack_map();
        std::mem::swap(&mut self.board.cached.active_king_pos, &mut self.board.cached.inactive_king_pos);
        self.board.active_colour = !self.board.active_colour;
        atk_map.contains(sq)
    }
}

const fn compute_pawn_moves() -> [[Bitboard; 64]; 2] {
    let mut black_squares = [Bitboard(0); 64];
    let mut white_squares = [Bitboard(0); 64];

    let mut index = 0;
    while index < 64 {
        let pos = Square(index);

        let num_up = pos.rank().0;
        let num_down = 7 - pos.rank().0;
        let num_left = pos.file().0;
        let num_right = 7 - pos.file().0;

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
        let pos = Square(index);

        let num_up = 7 - pos.rank().0;
        let num_down = pos.rank().0;
        let num_left = pos.file().0;
        let num_right = 7 - pos.file().0;

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
        let pos = Square(index);

        let num_up = 7 - pos.rank().0;
        let num_down = pos.rank().0;
        let num_left = pos.file().0;
        let num_right = 7 - pos.file().0;

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
        let pos = Square(index);

        let num_up = 7 - pos.rank().0;
        let num_down = pos.rank().0;
        let num_left = pos.file().0;
        let num_right = 7 - pos.file().0;

        squares[pos.0 as usize] = [
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
