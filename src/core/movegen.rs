use crate::prelude::*;
pub const KING_MOVES: [Bitboard; 64] = compute_king_moves();
pub const KNIGHT_MOVES: [Bitboard; 64] = compute_knight_moves();
pub const PAWN_ATTACKS: [[Bitboard; 64]; 2] = compute_pawn_moves();

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

fn gen_legal_moves<G: GenType>(board: &mut Board) -> Moves {
    let mut moves = gen_pseudolegal_moves::<G>(board);
    moves.retain(|mov| board.is_legal(*mov));
    moves
}

fn gen_pseudolegal_moves<G: GenType>(board: &Board) -> Moves {
    let pieces = board.friendly_bitboards();
    let all_pieces = board.all_pieces();
    let checkers = board.gen_checkers(board.active_side);
    let mut moves = Moves::new();
    if let Some(king_pos) = board.active_king() {
        gen_king_moves::<G>(board, king_pos, checkers, &mut moves);
    }
    if checkers.count() >= 2 {
        return moves;
    }
    gen_pawn_captures(board, &mut moves);
    if !G::CAPTURES_ONLY {
        gen_pawn_push(board, &mut moves);
    }
    pieces[Knight]
        .for_each(|from| push_squares::<G>(board, from, KNIGHT_MOVES[from as usize], &mut moves));
    pieces[Bishop].for_each(|from| {
        push_squares::<G>(board, from, bishop_attacks(from, all_pieces), &mut moves);
    });
    pieces[Rook].for_each(|from| {
        push_squares::<G>(board, from, rook_attacks(from, all_pieces), &mut moves);
    });
    pieces[Queen].for_each(|from| {
        push_squares::<G>(board, from, queen_attacks(from, all_pieces), &mut moves);
    });
    moves
}

fn gen_pawn_captures(board: &Board, moves: &mut Moves) {
    let promoting_row = (if board.active_side.is_white() { Rank::_7 } else { Rank::_2 }).mask();
    let pawns = board[Pawn] & board[board.active_side];
    let enemy_pieces = board[!board.active_side].shift_back(board.active_side);
    (enemy_pieces.shift_left() & pawns & promoting_row).for_each(|sq| {
        let to = unsafe { sq.add_rank_unchecked(board.active_side.forward()).add_int_unchecked(1) };
        moves.push(Move::new(sq, to, MoveFlags::QueenPromotionCapture));
        moves.push(Move::new(sq, to, MoveFlags::KnightPromotionCapture));
        moves.push(Move::new(sq, to, MoveFlags::BishopPromotionCapture));
        moves.push(Move::new(sq, to, MoveFlags::RookPromotionCapture));
    });
    (enemy_pieces.shift_right() & pawns & promoting_row).for_each(|sq| {
        let to = unsafe { sq.add_rank_unchecked(board.active_side.forward()).sub_int_unchecked(1) };
        moves.push(Move::new(sq, to, MoveFlags::QueenPromotionCapture));
        moves.push(Move::new(sq, to, MoveFlags::KnightPromotionCapture));
        moves.push(Move::new(sq, to, MoveFlags::BishopPromotionCapture));
        moves.push(Move::new(sq, to, MoveFlags::RookPromotionCapture));
    });
    (enemy_pieces.shift_left() & pawns & !promoting_row).for_each(|sq| {
        let to = unsafe { sq.add_rank_unchecked(board.active_side.forward()).add_int_unchecked(1) };
        moves.push(Move::new(sq, to, MoveFlags::Capture));
    });
    (enemy_pieces.shift_right() & pawns & !promoting_row).for_each(|sq| {
        let to = unsafe { sq.add_rank_unchecked(board.active_side.forward()).sub_int_unchecked(1) };
        moves.push(Move::new(sq, to, MoveFlags::Capture));
    });
    if let Some(en_passant) = board.en_passant_target_square {
        let en_passant_pawns = (en_passant.mask().shift_left() | en_passant.mask().shift_right())
            .shift_back(board.active_side)
            & pawns;
        en_passant_pawns.for_each(|sq| {
            moves.push(Move::new(sq, en_passant, MoveFlags::EnPassant));
        });
    }
}

fn gen_pawn_push(board: &Board, moves: &mut Moves) {
    let blocked_pawns = board.all_pieces().shift_forward(!board.active_side);
    let pawns = board[Pawn] & board[board.active_side] & !blocked_pawns;
    let promoting_row = (if board.active_side.is_white() { Rank::_7 } else { Rank::_2 }).mask();
    let double_move_row = (if board.active_side.is_white() { Rank::_2 } else { Rank::_7 }).mask();
    let promoting_pawns = pawns & promoting_row;
    let nonpromoting_pawns = pawns & !promoting_row;

    promoting_pawns.for_each(|sq| {
        for flags in [
            MoveFlags::QueenPromotion,
            MoveFlags::KnightPromotion,
            MoveFlags::BishopPromotion,
            MoveFlags::RookPromotion,
        ] {
            moves.push(Move::new(
                sq,
                unsafe { sq.add_rank_unchecked(board.active_side.forward()) },
                flags,
            ));
        }
    });

    let pawns2 = pawns & double_move_row & !blocked_pawns.shift_forward(!board.active_side);
    pawns2.for_each(|sq| {
        moves.push(Move::new(
            sq,
            unsafe { sq.add_rank_unchecked(board.active_side.forward() * 2) },
            MoveFlags::DoublePawnPush,
        ));
    });

    nonpromoting_pawns.for_each(|sq| {
        moves.push(Move::new(
            sq,
            unsafe { sq.add_rank_unchecked(board.active_side.forward()) },
            MoveFlags::Quiet,
        ));
    });
}

impl Board {
    #[must_use]
    pub fn pseudolegal_moves(&self) -> Moves {
        gen_pseudolegal_moves::<FullGen>(self)
    }

    #[must_use]
    pub fn legal_moves(&mut self) -> Moves {
        gen_legal_moves::<FullGen>(self)
    }

    #[must_use]
    pub fn pseudolegal_capture_moves(&self) -> Moves {
        gen_pseudolegal_moves::<CapturesOnly>(self)
    }

    #[must_use]
    pub fn capture_moves(&mut self) -> Moves {
        gen_legal_moves::<CapturesOnly>(self)
    }

    #[must_use]
    pub fn is_valid(&mut self, mov: Move) -> bool {
        self.pseudolegal_moves().contains(&mov) && self.is_legal(mov)
    }
}

fn push_squares<G: GenType>(board: &Board, from: Square, squares: Bitboard, moves: &mut Moves) {
    let captures = squares & board[!board.active_side];
    captures.for_each(|sq| moves.push(Move::new(from, sq, MoveFlags::Capture)));
    if !G::CAPTURES_ONLY {
        let noncaptures = squares & !board[!board.active_side] & !board[board.active_side];
        noncaptures.for_each(|sq| moves.push(Move::new(from, sq, MoveFlags::Quiet)));
    }
}

fn gen_king_moves<G: GenType>(board: &Board, from: Square, checkers: Bitboard, moves: &mut Moves) {
    push_squares::<G>(board, from, KING_MOVES[from as usize], moves);
    if G::CAPTURES_ONLY || !checkers.is_empty() {
        return;
    }
    if board.active_side == White {
        if board.can_castle.contains(CanCastle::WHITE_KING_SIDE)
            && !board.is_piece_at(Square::F1)
            && !board.is_piece_at(Square::G1)
        {
            moves.push(Move::new(from, Square::G1, MoveFlags::KingCastle));
        }
        if board.can_castle.contains(CanCastle::WHITE_QUEEN_SIDE)
            && !board.is_piece_at(Square::C1)
            && !board.is_piece_at(Square::D1)
            && !board.is_piece_at(Square::B1)
        {
            moves.push(Move::new(from, Square::C1, MoveFlags::QueenCastle));
        }
    } else {
        if board.can_castle.contains(CanCastle::BLACK_KING_SIDE)
            && !board.is_piece_at(Square::F8)
            && !board.is_piece_at(Square::G8)
        {
            moves.push(Move::new(from, Square::G8, MoveFlags::KingCastle));
        }
        if board.can_castle.contains(CanCastle::BLACK_QUEEN_SIDE)
            && !board.is_piece_at(Square::B8)
            && !board.is_piece_at(Square::C8)
            && !board.is_piece_at(Square::D8)
        {
            moves.push(Move::new(from, Square::C8, MoveFlags::QueenCastle));
        }
    }
}

impl Board {
    #[must_use]
    pub fn gen_checkers(&self, side: Side) -> Bitboard {
        let mut bb = Bitboard::EMPTY;
        let occupancy = self.all_pieces();
        let Some(king) = self.get_king_square(side) else { return bb };
        bb |= PAWN_ATTACKS[side as usize][king as usize] & self[Pawn];
        bb |= KNIGHT_MOVES[king as usize] & self[Knight];
        bb |= bishop_attacks(king, occupancy) & (self[Bishop] | self[Queen]);
        bb |= rook_attacks(king, occupancy) & (self[Rook] | self[Queen]);
        bb |= KING_MOVES[king as usize] & (self[King]);

        bb & self[!side]
    }

    #[must_use]
    /// Checks whether a move generated from `Board::pseudolegal_moves` is legal.
    ///
    /// Will not check if the move is entirely valid, use `Board::is_valid` instead
    pub fn is_legal(&mut self, mov: Move) -> bool {
        if mov.flags() == MoveFlags::KingCastle || mov.flags() == MoveFlags::QueenCastle {
            let map = self.gen_attack_map();
            let squares = match (self.active_side, mov.flags() == MoveFlags::KingCastle) {
                (Side::White, true) => [Square::F1, Square::G1],
                (Side::White, false) => [Square::C1, Square::D1],
                (Side::Black, true) => [Square::F8, Square::G8],
                (Side::Black, false) => [Square::C8, Square::D8],
            };
            if map.contains(squares[0]) || map.contains(squares[1]) {
                return false;
            }
        }
        let mut board = self.clone();
        board.make_move_no_update(mov);
        board.gen_checkers(self.active_side).is_empty()
    }

    // Generate attack map for enemy pieces
    fn gen_attack_map(&self) -> Bitboard {
        let mut output = Bitboard(0);
        let side = !self.active_side;
        let enemy_pieces = self.enemy_bitboards();
        let all_pieces = self.all_pieces();

        enemy_pieces[Pawn].for_each(|from| output |= PAWN_ATTACKS[side as usize][from as usize]);
        enemy_pieces[Knight].for_each(|from| output |= KNIGHT_MOVES[from as usize]);
        enemy_pieces[Bishop].for_each(|from| output |= bishop_attacks(from, all_pieces));
        enemy_pieces[Rook].for_each(|from| output |= rook_attacks(from, all_pieces));
        enemy_pieces[Queen].for_each(|from| output |= queen_attacks(from, all_pieces));

        if let Some(king) = self.inactive_king() {
            output |= KING_MOVES[king as usize];
        }
        output
    }

    #[must_use]
    pub fn pawn_attacks(&self, side: Side) -> Bitboard {
        let pawns = (self[side] & self[Pawn]).shift_forward(side);
        pawns.shift_left() | pawns.shift_right()
    }
}

const fn compute_pawn_moves() -> [[Bitboard; 64]; 2] {
    let mut black_squares = [Bitboard(0); 64];
    let mut white_squares = [Bitboard(0); 64];

    let mut index = 0;
    while index < 64 {
        let sq = Square::from_int(index as u8).unwrap();

        let num_up = sq.rank() as i8;
        let num_down = 7 - sq.rank() as i8;
        let num_left = sq.file() as i8;
        let num_right = 7 - sq.file() as i8;

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
        let sq = unsafe { Square::from_int_unchecked(index as u8) };

        let num_up = 7 - sq.rank() as i8;
        let num_down = sq.rank() as i8;
        let num_left = sq.file() as i8;
        let num_right = 7 - sq.file() as i8;

        let mut bitboard = Bitboard(0);

        let up = 8;
        let down = -up;
        let left = -1;
        let right = -left;

        bitboard.0 |= if num_up >= 2 && num_left >= 1 { 1 << (index + up * 2 + left) } else { 0 };
        bitboard.0 |= if num_up >= 2 && num_right >= 1 { 1 << (index + up * 2 + right) } else { 0 };
        bitboard.0 |=
            if num_down >= 2 && num_left >= 1 { 1 << (index + down * 2 + left) } else { 0 };
        bitboard.0 |=
            if num_down >= 2 && num_right >= 1 { 1 << (index + down * 2 + right) } else { 0 };

        bitboard.0 |= if num_left >= 2 && num_up >= 1 { 1 << (index + up + left * 2) } else { 0 };
        bitboard.0 |= if num_right >= 2 && num_up >= 1 { 1 << (index + up + right * 2) } else { 0 };
        bitboard.0 |=
            if num_left >= 2 && num_down >= 1 { 1 << (index + down + left * 2) } else { 0 };
        bitboard.0 |=
            if num_right >= 2 && num_down >= 1 { 1 << (index + down + right * 2) } else { 0 };

        squares[index as usize] = bitboard;
        index += 1;
    }
    squares
}

const fn compute_king_moves() -> [Bitboard; 64] {
    let mut squares = [Bitboard(0); 64];

    let mut index = 0;
    while index < 64 {
        let sq = Square::from_int(index as u8).unwrap();

        let num_up = 7 - sq.rank() as i8;
        let num_down = sq.rank() as i8;
        let num_left = sq.file() as i8;
        let num_right = 7 - sq.file() as i8;

        let mut bitboard = Bitboard(0);

        let up = 8i32;
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
