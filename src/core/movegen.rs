use super::magic::Magic;
use crate::prelude::*;
pub const DIRECTION_OFFSETS: [i8; 8] = [8, -8, -1, 1, 7, -7, 9, -9];
pub const NUM_SQUARES_TO_EDGE: [[i8; 8]; 64] = compute_num_squares_to_edge();
const KING_MOVES: [Bitboard; 64] = compute_king_moves();
const KNIGHT_MOVES: [Bitboard; 64] = compute_knight_moves();
const ATTACK_PAWN_MOVES: [[Bitboard; 64]; 2] = compute_pawn_moves();

pub struct MoveGenerator<'a> {
    moves: Moves,
    board: &'a mut Board,
    captures_only: bool,
    attacked_squares: Option<Bitboard>,
    pub queen_promote_only: bool,
    magic: &'static Magic,
}

impl Board {
    #[must_use]
    pub fn gen_pseudolegal_moves(&mut self) -> Moves {
        MoveGenerator::new(self).gen_pseudolegal_moves()
    }
    #[must_use]
    pub fn gen_legal_moves(&mut self) -> Moves {
        let mut movegen = MoveGenerator::new(self);
        movegen.queen_promote_only = false;
        movegen.gen_legal_moves()
    }
    #[must_use]
    pub fn gen_capture_moves(&mut self) -> Moves {
        let mut movegen = MoveGenerator::new(self);
        movegen.captures_only = true;
        movegen.gen_legal_moves()
    }
    #[must_use]
    pub fn gen_pseudolegal_capture_moves(&mut self) -> Moves {
        let mut movegen = MoveGenerator::new(self);
        movegen.captures_only = true;
        movegen.gen_pseudolegal_moves()
    }
}

impl<'a> MoveGenerator<'a> {
    #[must_use]
    pub fn new(board: &'a mut Board) -> Self {
        Self {
            moves: Moves::default(),
            board,
            captures_only: false,
            attacked_squares: None,
            queen_promote_only: true,
            magic: Magic::get(),
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
        pieces[Pawn].for_each(|from| self.gen_pawn_moves(from));
        pieces[Knight].for_each(|from| self.gen_knight_moves(from));
        pieces[Bishop].for_each(|from| self.gen_sliding_moves(from, Bishop + self.board.active_colour));
        pieces[Rook].for_each(|from| self.gen_sliding_moves(from, Rook + self.board.active_colour));
        pieces[Queen].for_each(|from| self.gen_sliding_moves(from, Queen + self.board.active_colour));
        self.gen_king_moves(self.board.active_king_pos);
        std::mem::take(&mut self.moves)
    }
    #[must_use]
    #[inline]
    pub fn is_legal(&mut self, mov: Move) -> bool {
        if mov.flags() == MoveFlags::KingCastle || mov.flags() == MoveFlags::QueenCastle {
            let map = self.attack_map();
            let squares = match (self.board.active_colour, mov.flags() == MoveFlags::KingCastle) {
                (Colour::White, true) => [Pos::F1, Pos::G1],
                (Colour::White, false) => [Pos::C1, Pos::D1],
                (Colour::Black, true) => [Pos::F8, Pos::G8],
                (Colour::Black, false) => [Pos::C8, Pos::D8],
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
        enemy_pieces[Queen].for_each(|from| {
            attacked_squares |= self.magic.bishop_attacks(from, all_pieces);
            attacked_squares |= self.magic.rook_attacks(from, all_pieces);
        });
        attacked_squares |= KING_MOVES[self.board.inactive_king_pos];
        attacked_squares
    }

    #[allow(clippy::needless_range_loop)]
    fn gen_sliding_moves(&mut self, from: Pos, piece: Piece) {
        let mut squares = match piece.kind() {
            PieceKind::Bishop => self.magic.bishop_attacks(from, self.board.all_pieces()),
            PieceKind::Rook => self.magic.rook_attacks(from, self.board.all_pieces()),
            PieceKind::Queen => {
                self.magic.bishop_attacks(from, self.board.all_pieces())
                    | self.magic.rook_attacks(from, self.board.all_pieces())
            }
            _ => unreachable!(),
        };
        squares.0 &= !self.board.friendly_pieces().0;

        squares.for_each(|square| {
            if self.board[square].is_some() {
                self.moves.push(Move::new(from, square, MoveFlags::Capture));
            } else if !self.captures_only {
                self.moves.push(Move::new(from, square, MoveFlags::Quiet));
            }
        });
    }
    fn gen_pawn_moves(&mut self, from: Pos) {
        let forward = self.board.active_colour.forward();

        let can_promote = (self.board.white_to_play() && from.rank().0 == 6)
            || (self.board.black_to_play() && from.rank().0 == 1);

        if !self.captures_only {
            let to = Pos(from.0 + forward * 8);
            if self.board[to].is_none() {
                let can_double_push = (self.board.white_to_play() && from.rank().0 == 1)
                    || (self.board.black_to_play() && from.rank().0 == 6);

                if !can_promote {
                    self.moves.push(Move::new(from, to, MoveFlags::Quiet));
                }

                if can_double_push {
                    let to = Pos(from.0 + forward * 16);
                    if self.board[to].is_none() {
                        self.moves.push(Move::new(from, to, MoveFlags::DoublePawnPush));
                    }
                } else if can_promote {
                    self.moves.push(Move::new(from, to, MoveFlags::QueenPromotion));
                    if !self.queen_promote_only {
                        self.moves.push(Move::new(from, to, MoveFlags::KnightPromotion));
                        self.moves.push(Move::new(from, to, MoveFlags::BishopPromotion));
                        self.moves.push(Move::new(from, to, MoveFlags::RookPromotion));
                    }
                }
            }
        }
        if let Some(to) = Pos(from.0 + forward * 8).add_file(1) {
            if self.board[to].map(Piece::colour) == Some(!self.board.active_colour) {
                if can_promote {
                    self.moves.push(Move::new(from, to, MoveFlags::QueenPromotionCapture));
                    if !self.queen_promote_only {
                        self.moves.push(Move::new(from, to, MoveFlags::KnightPromotionCapture));
                        self.moves.push(Move::new(from, to, MoveFlags::BishopPromotionCapture));
                        self.moves.push(Move::new(from, to, MoveFlags::RookPromotionCapture));
                    }
                } else {
                    self.moves.push(Move::new(from, to, MoveFlags::Capture));
                }
            }
        }
        if let Some(to) = Pos(from.0 + forward * 8).add_file(-1) {
            if self.board[to].map(Piece::colour) == Some(!self.board.active_colour) {
                if can_promote {
                    if !self.queen_promote_only {
                        self.moves.push(Move::new(from, to, MoveFlags::KnightPromotionCapture));
                        self.moves.push(Move::new(from, to, MoveFlags::BishopPromotionCapture));
                        self.moves.push(Move::new(from, to, MoveFlags::RookPromotionCapture));
                    }
                    self.moves.push(Move::new(from, to, MoveFlags::QueenPromotionCapture));
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
    fn gen_knight_moves(&mut self, from: Pos) {
        let bitboard = KNIGHT_MOVES[from] & !self.board.friendly_pieces();
        bitboard.for_each(|to| {
            if self.board[to].is_some() {
                self.moves.push(Move::new(from, to, MoveFlags::Capture));
            } else if !self.captures_only {
                self.moves.push(Move::new(from, to, MoveFlags::Quiet));
            }
        });
    }
    #[allow(clippy::needless_range_loop)]
    fn gen_king_moves(&mut self, from: Pos) {
        let bitboard = KING_MOVES[from] & !self.board.friendly_pieces();
        bitboard.for_each(|to| {
            if self.board[to].is_some() {
                self.moves.push(Move::new(from, to, MoveFlags::Capture));
            } else if !self.captures_only {
                self.moves.push(Move::new(from, to, MoveFlags::Quiet));
            }
        });
        if self.captures_only {
            return;
        }
        if self.board.white_to_play() {
            if self.board.can_castle.contains(CanCastle::WHITE_KING_SIDE)
                && self.board[Pos::F1].is_none()
                && self.board[Pos::G1].is_none()
            {
                self.moves.push(Move::new(from, Pos::G1, MoveFlags::KingCastle));
            }
            if self.board.can_castle.contains(CanCastle::WHITE_QUEEN_SIDE)
                && self.board[Pos::C1].is_none()
                && self.board[Pos::D1].is_none()
                && self.board[Pos::B1].is_none()
            {
                self.moves.push(Move::new(from, Pos::C1, MoveFlags::QueenCastle));
            }
        } else {
            if self.board.can_castle.contains(CanCastle::BLACK_KING_SIDE)
                && self.board[Pos::F8].is_none()
                && self.board[Pos::G8].is_none()
            {
                self.moves.push(Move::new(from, Pos::G8, MoveFlags::KingCastle));
            }
            if self.board.can_castle.contains(CanCastle::BLACK_QUEEN_SIDE)
                && self.board[Pos::B8].is_none()
                && self.board[Pos::C8].is_none()
                && self.board[Pos::D8].is_none()
            {
                self.moves.push(Move::new(from, Pos::C8, MoveFlags::QueenCastle));
            }
        }
    }
}

impl<'a> MoveGenerator<'a> {
    #[inline]
    pub fn is_square_attacked(&mut self, square: Pos) -> bool {
        self.board.active_colour = !self.board.active_colour;
        std::mem::swap(&mut self.board.cached.active_king_pos, &mut self.board.cached.inactive_king_pos);
        let atk_map = self.gen_attack_map();
        std::mem::swap(&mut self.board.cached.active_king_pos, &mut self.board.cached.inactive_king_pos);
        self.board.active_colour = !self.board.active_colour;
        atk_map.contains(square)
    }
}

const fn compute_pawn_moves() -> [[Bitboard; 64]; 2] {
    let mut black_squares = [Bitboard(0); 64];
    let mut white_squares = [Bitboard(0); 64];

    let mut index = 0;
    while index < 64 {
        let pos = Pos(index);

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
        let pos = Pos(index);

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
        let pos = Pos(index);

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
        let pos = Pos(index);

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
