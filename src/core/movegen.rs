use crate::prelude::*;
pub const KING_MOVES: [Bitboard; 64] = compute_king_moves();
pub const KNIGHT_MOVES: [Bitboard; 64] = compute_knight_moves();
pub const PAWN_ATTACKS: [[Bitboard; 64]; 2] = compute_pawn_moves();

pub struct QuietOnly;
pub struct CapturesOnly;
pub struct FullGen;

pub trait GenType {
    const CAPTURES: bool;
    const NONCAPTURES: bool;
}

impl GenType for QuietOnly {
    const CAPTURES: bool = false;
    const NONCAPTURES: bool = true;
}

impl GenType for CapturesOnly {
    const CAPTURES: bool = true;
    const NONCAPTURES: bool = false;
}

impl GenType for FullGen {
    const CAPTURES: bool = true;
    const NONCAPTURES: bool = true;
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
    if G::CAPTURES {
        gen_pawn_captures(board, &mut moves);
    }
    if G::NONCAPTURES {
        gen_pawn_push(board, &mut moves);
    }
    for from in pieces[Knight] {
        push_squares::<G>(board, from, KNIGHT_MOVES[from], &mut moves);
    }
    for from in pieces[Bishop] {
        push_squares::<G>(board, from, bishop_attacks(from, all_pieces), &mut moves);
    }
    for from in pieces[Rook] {
        push_squares::<G>(board, from, rook_attacks(from, all_pieces), &mut moves);
    }
    for from in pieces[Queen] {
        push_squares::<G>(board, from, queen_attacks(from, all_pieces), &mut moves);
    }
    moves
}

fn gen_pawn_captures(board: &Board, moves: &mut Moves) {
    let promoting_row = (if board.active_side.is_white() { Rank::_7 } else { Rank::_2 }).mask();
    let pawns = board[Pawn] & board[board.active_side];
    let enemy_pieces = board[!board.active_side].shift_back(board.active_side);
    for sq in enemy_pieces.shift_left() & pawns & promoting_row {
        let to = unsafe { sq.add_rank_unchecked(board.active_side.forward()).add_int_unchecked(1) };
        moves.push(Move::new(sq, to, MoveFlags::QueenPromotionCapture));
        moves.push(Move::new(sq, to, MoveFlags::KnightPromotionCapture));
        moves.push(Move::new(sq, to, MoveFlags::BishopPromotionCapture));
        moves.push(Move::new(sq, to, MoveFlags::RookPromotionCapture));
    }
    for sq in enemy_pieces.shift_right() & pawns & promoting_row {
        let to = unsafe { sq.add_rank_unchecked(board.active_side.forward()).sub_int_unchecked(1) };
        moves.push(Move::new(sq, to, MoveFlags::QueenPromotionCapture));
        moves.push(Move::new(sq, to, MoveFlags::KnightPromotionCapture));
        moves.push(Move::new(sq, to, MoveFlags::BishopPromotionCapture));
        moves.push(Move::new(sq, to, MoveFlags::RookPromotionCapture));
    }
    for sq in enemy_pieces.shift_left() & pawns & !promoting_row {
        let to = unsafe { sq.add_rank_unchecked(board.active_side.forward()).add_int_unchecked(1) };
        moves.push(Move::new(sq, to, MoveFlags::Capture));
    }
    for sq in enemy_pieces.shift_right() & pawns & !promoting_row {
        let to = unsafe { sq.add_rank_unchecked(board.active_side.forward()).sub_int_unchecked(1) };
        moves.push(Move::new(sq, to, MoveFlags::Capture));
    }
    if let Some(en_passant) = board.en_passant_target_square {
        let en_passant_pawns = (en_passant.mask().shift_left() | en_passant.mask().shift_right())
            .shift_back(board.active_side)
            & pawns;
        for sq in en_passant_pawns {
            moves.push(Move::new(sq, en_passant, MoveFlags::EnPassant));
        }
    }
}

fn gen_pawn_push(board: &Board, moves: &mut Moves) {
    let blocked_pawns = board.all_pieces().shift_forward(!board.active_side);
    let pawns = board[Pawn] & board[board.active_side] & !blocked_pawns;
    let promoting_row = (if board.active_side.is_white() { Rank::_7 } else { Rank::_2 }).mask();
    let double_move_row = (if board.active_side.is_white() { Rank::_2 } else { Rank::_7 }).mask();
    let promoting_pawns = pawns & promoting_row;
    let nonpromoting_pawns = pawns & !promoting_row;

    for sq in promoting_pawns {
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
    }

    let pawns2 = pawns & double_move_row & !blocked_pawns.shift_forward(!board.active_side);
    for sq in pawns2 {
        moves.push(Move::new(
            sq,
            unsafe { sq.add_rank_unchecked(board.active_side.forward() * 2) },
            MoveFlags::DoublePawnPush,
        ));
    }

    for sq in nonpromoting_pawns {
        moves.push(Move::new(
            sq,
            unsafe { sq.add_rank_unchecked(board.active_side.forward()) },
            MoveFlags::Quiet,
        ));
    }
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
    pub fn pseudolegal_quiet_moves(&self) -> Moves {
        gen_pseudolegal_moves::<QuietOnly>(self)
    }

    #[must_use]
    pub fn capture_moves(&mut self) -> Moves {
        gen_legal_moves::<CapturesOnly>(self)
    }

    #[must_use]
    pub fn is_valid(&mut self, mov: Move) -> bool {
        self.pseudolegal_moves().contains(&mov) && self.is_legal(mov)
    }

    pub(crate) fn is_pseudolegal(&mut self, mov: Move) -> bool {
        let result = self.is_pseudolegal_(mov);
        debug_assert_eq!(result, self.pseudolegal_moves().contains(&mov), "{self:?} - {mov:?}");
        result
    }

    #[must_use]
    #[expect(clippy::too_many_lines)]
    pub(crate) fn is_pseudolegal_(&mut self, mov: Move) -> bool {
        let (from, to) = (mov.from(), mov.to());

        let Some(from_piece) = self.get_square(from) else { return false };
        if from_piece.side() != self.active_side {
            return false;
        }

        let checkers = self.gen_checkers(self.active_side);

        if checkers.count() >= 2 && from_piece.kind() != King {
            return false;
        }

        if mov.flags() == MoveFlags::KingCastle || mov.flags() == MoveFlags::QueenCastle {
            let can_castle = match (self.active_side, mov.flags() == MoveFlags::KingCastle) {
                (White, true) => CanCastle::WHITE_KING_SIDE,
                (White, false) => CanCastle::WHITE_QUEEN_SIDE,
                (Black, true) => CanCastle::BLACK_KING_SIDE,
                (Black, false) => CanCastle::BLACK_QUEEN_SIDE,
            };
            if !self.can_castle.contains(can_castle) {
                return false;
            }
            if !checkers.is_empty() {
                return false;
            }
            let squares = match (self.active_side, mov.flags() == MoveFlags::KingCastle) {
                (White, true) => Bitboard::from_iter([Square::F1, Square::G1]),
                (White, false) => Bitboard::from_iter([Square::B1, Square::C1, Square::D1]),
                (Black, true) => Bitboard::from_iter([Square::F8, Square::G8]),
                (Black, false) => Bitboard::from_iter([Square::B8, Square::C8, Square::D8]),
            };
            return (squares & self.all_pieces()).is_empty();
        }

        let mut en_passant = false;
        if from_piece.kind() == Pawn
            && let Some(sq) = self.en_passant_target_square
            && mov.to() == sq
        {
            if !PAWN_ATTACKS[!self.active_side][sq].contains(from) {
                return false;
            }
            en_passant = true;
        }

        if en_passant != (mov.flags() == MoveFlags::EnPassant) {
            return false;
        }

        let captured_piece = self.get_square(to);
        if let Some(captured_piece) = captured_piece {
            if captured_piece.side() == self.active_side {
                return false;
            }
            if !mov.flags().is_capture() {
                return false;
            }
        } else if mov.flags().is_capture() && !en_passant {
            return false;
        }

        if from_piece.kind() == Pawn
            && ((self.active_side.is_white() && to.rank() == Rank::_8)
                || (self.active_side.is_black() && to.rank() == Rank::_1))
            && mov.flags().promotion().is_none()
        {
            return false;
        }

        if mov.flags().promotion().is_some() && from_piece.kind() != Pawn {
            return false;
        }
        if (mov.flags() == MoveFlags::KingCastle || mov.flags() == MoveFlags::QueenCastle)
            && from_piece.kind() != King
        {
            return false;
        }

        let move_options = match from_piece.kind() {
            Pawn => {
                if captured_piece.is_some() {
                    PAWN_ATTACKS[self.active_side][from]
                } else if en_passant {
                    return true;
                } else {
                    let forward = from.add_rank(self.active_side.forward()).unwrap();
                    if forward == to {
                        return true;
                    }
                    if mov.flags() != MoveFlags::DoublePawnPush {
                        return false;
                    }

                    if ((self.active_side.is_white() && from.rank() == Rank::_2)
                        || (self.active_side.is_black() && from.rank() == Rank::_7))
                        && !self.is_piece_at(forward)
                    {
                        let forward2 = forward.add_rank(self.active_side.forward()).unwrap();
                        if forward2 == to {
                            return true;
                        }
                    }
                    return false;
                }
            }
            Knight => KNIGHT_MOVES[from],
            Bishop => bishop_attacks(from, self.all_pieces()),
            Rook => rook_attacks(from, self.all_pieces()),
            Queen => queen_attacks(from, self.all_pieces()),
            King => KING_MOVES[from],
        };
        if !move_options.contains(mov.to()) {
            return false;
        }
        if mov.flags() == MoveFlags::DoublePawnPush {
            return false;
        }
        true
    }
}

fn push_squares<G: GenType>(board: &Board, from: Square, squares: Bitboard, moves: &mut Moves) {
    if G::CAPTURES {
        let captures = squares & board[!board.active_side];
        for sq in captures {
            moves.push(Move::new(from, sq, MoveFlags::Capture));
        }
    }
    if G::NONCAPTURES {
        let noncaptures = squares & !board.all_pieces();
        for sq in noncaptures {
            moves.push(Move::new(from, sq, MoveFlags::Quiet));
        }
    }
}

fn gen_king_moves<G: GenType>(board: &Board, from: Square, checkers: Bitboard, moves: &mut Moves) {
    push_squares::<G>(board, from, KING_MOVES[from], moves);
    if !G::NONCAPTURES || !checkers.is_empty() {
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
    pub fn gen_attackers(&self, sq: Square, side: Side) -> Bitboard {
        let mut bb = Bitboard::EMPTY;
        let occupancy = self.all_pieces();
        bb |= PAWN_ATTACKS[side][sq] & self[Pawn];
        bb |= KNIGHT_MOVES[sq] & self[Knight];
        bb |= bishop_attacks(sq, occupancy) & (self[Bishop] | self[Queen]);
        bb |= rook_attacks(sq, occupancy) & (self[Rook] | self[Queen]);
        bb |= KING_MOVES[sq] & (self[King]);

        bb & self[!side]
    }

    #[must_use]
    pub fn gen_checkers(&self, side: Side) -> Bitboard {
        let Some(king) = self.get_king_square(side) else { return Bitboard::EMPTY };
        self.gen_attackers(king, side)
    }

    #[must_use]
    /// Checks whether a move generated from `Board::pseudolegal_moves` is legal.
    ///
    /// Will not check if the move is entirely valid, use `Board::is_valid` instead
    pub fn is_legal(&mut self, mov: Move) -> bool {
        if mov.flags() == MoveFlags::KingCastle || mov.flags() == MoveFlags::QueenCastle {
            std::hint::cold_path();
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
        let enemy_pieces = self.enemy_bitboards();
        let all_pieces = self.all_pieces();

        output |= self.pawn_attacks(!self.active_side);
        for from in enemy_pieces[Knight] {
            output |= KNIGHT_MOVES[from];
        }
        for from in enemy_pieces[Bishop] {
            output |= bishop_attacks(from, all_pieces);
        }
        for from in enemy_pieces[Rook] {
            output |= rook_attacks(from, all_pieces);
        }
        for from in enemy_pieces[Queen] {
            output |= queen_attacks(from, all_pieces);
        }

        if let Some(king) = self.inactive_king() {
            output |= KING_MOVES[king];
        } else {
            std::hint::cold_path();
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

#[test]
fn test_pawn_attacks() {
    assert_eq!(Board::start_pos().pawn_attacks(White), Rank::_3.mask());
}
