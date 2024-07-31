use crate::prelude::*;

const DIRECTION_OFFSETS: [i8; 8] = [8, -8, -1, 1, 7, -7, 9, -9];
static NUM_SQUARES_TO_EDGE: [[i8; 8]; 64] = compute_num_squares_to_edge();

pub struct MoveGenerator<'a> {
    moves: Moves,
    is_pseudolegal: bool,
    board: &'a mut Board,
    pub captures_only: bool,
    pub attacked_squares: Option<Bitboard>,
}

impl Board {
    #[must_use]
    pub fn gen_pseudolegal_moves(&mut self) -> Moves {
        MoveGenerator::new_pseudo_legal(self).gen_moves()
    }
    #[must_use]
    pub fn gen_legal_moves(&mut self) -> Moves {
        MoveGenerator::new(self).gen_moves()
    }
    #[must_use]
    pub fn gen_capture_moves(&mut self) -> Moves {
        let mut movegen = MoveGenerator::new(self);
        movegen.captures_only = true;
        movegen.gen_moves()
    }
}

impl<'a> MoveGenerator<'a> {
    #[must_use]
    pub fn new_pseudo_legal(board: &'a mut Board) -> Self {
        Self {
            moves: Moves::default(),
            board,
            is_pseudolegal: true,
            captures_only: false,
            attacked_squares: None,
        }
    }
    #[must_use]
    pub fn new(board: &'a mut Board) -> Self {
        Self {
            moves: Moves::default(),
            board,
            is_pseudolegal: false,
            captures_only: false,
            attacked_squares: None,
        }
    }
    #[must_use]
    pub fn gen_moves(&mut self) -> Moves {
        let mut moves = self.pseudolegal_moves();
        if !self.is_pseudolegal {
            moves.retain(|&mut mov| {
                let unmake = self.board.make_move(mov);
                let king_pos = self.board.inactive_king_pos;
                let is_attacked = self.is_square_attacked(king_pos);
                self.board.unmake_move(unmake);
                !is_attacked
            });
        }
        moves
    }
    pub fn attack_map(&mut self) -> Bitboard {
        if self.attacked_squares.is_none() {
            self.gen_attack_map();
        }
        self.attacked_squares.unwrap()
    }
    // Generate attack map for enemy pieces
    #[allow(clippy::needless_range_loop)]
    pub fn gen_attack_map(&mut self) {
        let forward = -self.board.active_colour.forward();
        let mut attacked_squares = Bitboard(0);

        for (from, piece) in self.board.piece_positions() {
            if piece.colour() == self.board.active_colour {
                continue;
            }
            match piece.kind() {
                PieceKind::Pawn => {
                    if let Some(to) = Pos(from.0 + forward * 8).add_file(1) {
                        attacked_squares.insert(to);
                    }
                    if let Some(to) = Pos(from.0 + forward * 8).add_file(-1) {
                        attacked_squares.insert(to);
                    }
                }
                PieceKind::Knight => {
                    for (file, rank) in
                        [(1, 2), (1, -2), (-1, 2), (-1, -2), (2, 1), (2, -1), (-2, 1), (-2, -1)]
                    {
                        if let Some(to) = from.add_file(file).and_then(|from| from.add_rank(rank)) {
                            attacked_squares.insert(to);
                        };
                    }
                }
                PieceKind::Bishop | PieceKind::Rook | PieceKind::Queen => {
                    let start_index = if piece.kind() == Bishop { 4 } else { 0 };
                    let end_index = if piece.kind() == Rook { 4 } else { 8 };

                    for direction_index in start_index..end_index {
                        for n in 0..NUM_SQUARES_TO_EDGE[from.0 as usize][direction_index] {
                            let target_square = Pos(from.0 + DIRECTION_OFFSETS[direction_index] * (n + 1));
                            attacked_squares.insert(target_square);
                            if self.board[target_square].map(Piece::colour).is_some() {
                                break;
                            }
                        }
                    }
                }
                PieceKind::King => {
                    for direction_index in 0..8 {
                        if NUM_SQUARES_TO_EDGE[from.0 as usize][direction_index] > 0 {
                            let target_square = Pos((from.0) + DIRECTION_OFFSETS[direction_index]);
                            attacked_squares.insert(target_square);
                        }
                    }
                }
            }
        }
        self.attacked_squares = Some(attacked_squares);
    }
    fn pseudolegal_moves(&mut self) -> Moves {
        for from in (0..64).map(Pos) {
            let Some(piece) = self.board[from] else { continue };
            if piece.colour() != self.board.active_colour {
                continue;
            }
            match piece.kind() {
                PieceKind::Bishop | PieceKind::Rook | PieceKind::Queen => {
                    self.gen_sliding_moves(from, piece);
                }
                PieceKind::Pawn => self.gen_pawn_moves(from),
                PieceKind::Knight => self.gen_knight_moves(from),
                PieceKind::King => self.gen_king_moves(from),
            }
        }
        std::mem::take(&mut self.moves)
    }
    #[allow(clippy::needless_range_loop)]
    fn gen_sliding_moves(&mut self, from: Pos, piece: Piece) {
        let start_index = if piece.kind() == Bishop { 4 } else { 0 };
        let end_index = if piece.kind() == Rook { 4 } else { 8 };

        for direction_index in start_index..end_index {
            for n in 0..NUM_SQUARES_TO_EDGE[from.0 as usize][direction_index] {
                let target_square = Pos(from.0 + DIRECTION_OFFSETS[direction_index] * (n + 1));
                let target_piece = self.board[target_square];

                if target_piece.map(Piece::colour) == Some(self.board.active_colour) {
                    break;
                }

                if target_piece.map(Piece::colour) == Some(!self.board.active_colour) {
                    self.moves.push(Move::new(from, target_square, MoveFlags::Capture));
                    break;
                }
                if !self.captures_only {
                    self.moves.push(Move::new(from, target_square, MoveFlags::Quiet));
                }
            }
        }
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
                    self.moves.push(Move::new(from, to, MoveFlags::KnightPromotion));
                    self.moves.push(Move::new(from, to, MoveFlags::BishopPromotion));
                    self.moves.push(Move::new(from, to, MoveFlags::RookPromotion));
                    self.moves.push(Move::new(from, to, MoveFlags::QueenPromotion));
                }
            }
        }
        if let Some(to) = Pos(from.0 + forward * 8).add_file(1) {
            if self.board[to].map(Piece::colour) == Some(!self.board.active_colour) {
                if can_promote {
                    self.moves.push(Move::new(from, to, MoveFlags::KnightPromotionCapture));
                    self.moves.push(Move::new(from, to, MoveFlags::BishopPromotionCapture));
                    self.moves.push(Move::new(from, to, MoveFlags::RookPromotionCapture));
                    self.moves.push(Move::new(from, to, MoveFlags::QueenPromotionCapture));
                } else {
                    self.moves.push(Move::new(from, to, MoveFlags::Capture));
                }
            }
        }
        if let Some(to) = Pos(from.0 + forward * 8).add_file(-1) {
            if self.board[to].map(Piece::colour) == Some(!self.board.active_colour) {
                if can_promote {
                    self.moves.push(Move::new(from, to, MoveFlags::KnightPromotionCapture));
                    self.moves.push(Move::new(from, to, MoveFlags::BishopPromotionCapture));
                    self.moves.push(Move::new(from, to, MoveFlags::RookPromotionCapture));
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
        for (file, rank) in [(1, 2), (1, -2), (-1, 2), (-1, -2), (2, 1), (2, -1), (-2, 1), (-2, -1)] {
            let Some(to) = from.add_file(file).and_then(|from| from.add_rank(rank)) else { continue };
            let target_colour = self.board[to].map(Piece::colour);

            if target_colour.is_none() {
                if !self.captures_only {
                    self.moves.push(Move::new(from, to, MoveFlags::Quiet));
                }
            } else if target_colour == Some(!self.board.active_colour) {
                self.moves.push(Move::new(from, to, MoveFlags::Capture));
            }
        }
    }
    #[allow(clippy::needless_range_loop)]
    fn gen_king_moves(&mut self, from: Pos) {
        for direction_index in 0..8 {
            if NUM_SQUARES_TO_EDGE[from.0 as usize][direction_index] == 0 {
                continue;
            }
            let target_square = Pos((from.0) + DIRECTION_OFFSETS[direction_index]);
            let target_piece = self.board[target_square];

            if target_piece.map(Piece::colour) == Some(self.board.active_colour) {
                continue;
            }

            if target_piece.map(Piece::colour) == Some(!self.board.active_colour) {
                self.moves.push(Move::new(from, target_square, MoveFlags::Capture));
                continue;
            }
            if !self.captures_only {
                self.moves.push(Move::new(from, target_square, MoveFlags::Quiet));
            }
        }
        if self.captures_only {
            return;
        }
        if self.board.white_to_play() {
            if self.board.can_castle.contains(CanCastle::WHITE_KING_SIDE)
                && self.can_castle_through([Pos::F1, Pos::G1])
                && self.board[Pos::E2] != Some(Black + Pawn)
            {
                self.moves.push(Move::new(from, Pos::G1, MoveFlags::KingCastle));
            }
            if self.board.can_castle.contains(CanCastle::WHITE_QUEEN_SIDE)
                && self.can_castle_through([Pos::C1, Pos::D1])
                && self.board[Pos::B1].is_none()
                && self.board[Pos::E2] != Some(Black + Pawn)
            {
                self.moves.push(Move::new(from, Pos::C1, MoveFlags::QueenCastle));
            }
        } else {
            if self.board.can_castle.contains(CanCastle::BLACK_KING_SIDE)
                && self.can_castle_through([Pos::F8, Pos::G8])
                && self.board[Pos::E7] != Some(White + Pawn)
            {
                self.moves.push(Move::new(from, Pos::G8, MoveFlags::KingCastle));
            }
            if self.board.can_castle.contains(CanCastle::BLACK_QUEEN_SIDE)
                && self.can_castle_through([Pos::C8, Pos::D8])
                && self.board[Pos::B8].is_none()
                && self.board[Pos::E7] != Some(White + Pawn)
            {
                self.moves.push(Move::new(from, Pos::C8, MoveFlags::QueenCastle));
            }
        }
    }
}

impl<'a> MoveGenerator<'a> {
    fn can_castle_through(&mut self, squares: [Pos; 2]) -> bool {
        if self.board[squares[0]].is_some() || self.board[squares[1]].is_some() {
            return false;
        }
        if self.is_pseudolegal {
            return true;
        }
        let map = self.attack_map();
        !(map.contains(squares[0]) || map.contains(squares[1]) || map.contains(self.board.active_king_pos))
    }
    pub(crate) fn is_square_attacked(&mut self, square: Pos) -> bool {
        let mut movegen = MoveGenerator {
            board: self.board,
            moves: Moves::new(),
            is_pseudolegal: true,
            captures_only: true,
            attacked_squares: None,
        };
        let moves = movegen.pseudolegal_moves();
        moves.into_iter().any(|mov| mov.to() == square)
    }
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

#[test]
fn perft_start() {
    let results = [1, 20, 400, 8_902, 197_281, 4_865_609, 119_060_324];
    for (depth, &result) in results.iter().enumerate() {
        let count = perft(&mut Board::start_pos(), depth as u8);
        assert_eq!(count, result, "depth: {depth}");
    }
}

#[test]
fn perft_kiwi() {
    let results = [1, 48, 2_039, 97_862, 4_085_603, 193_690_690];
    for (depth, &result) in results.iter().enumerate() {
        let count = perft(&mut Board::kiwipete(), depth as u8);
        assert_eq!(count, result, "depth: {depth}");
    }
}
#[test]
fn perft_position_3() {
    let results = [1, 14, 191, 2_812, 43_238, 674_624, 11_030_083];
    for (depth, &result) in results.iter().enumerate() {
        let count = perft(&mut Board::perft_position_3(), depth as u8);
        assert_eq!(count, result, "depth: {depth}");
    }
}

#[test]
fn perft_position_4() {
    let results = [1, 6, 264, 9_467, 422_333, 15_833_292 /*706_045_033*/];
    for (depth, &result) in results.iter().enumerate() {
        let count = perft(&mut Board::perft_position_4(), depth as u8);
        assert_eq!(count, result, "depth: {depth}");
    }
}

#[test]
fn perft_talk() {
    let results = [1, 44, 1_486, 62_379, 2_103_487, 89_941_194];
    for (depth, &result) in results.iter().enumerate() {
        let count = perft(&mut Board::perft_position_5(), depth as u8);
        assert_eq!(count, result, "depth: {depth}");
    }
}

#[cfg(test)]
fn perft(board: &mut Board, depthleft: u8) -> u64 {
    if depthleft == 0 {
        return 1;
    } else if depthleft == 1 {
        return board.gen_legal_moves().len() as u64;
    }
    let mut count = 0;
    for mov in board.gen_legal_moves() {
        let unmake = board.make_move(mov);
        count += perft(board, depthleft - 1);
        board.unmake_move(unmake);
    }
    count
}
