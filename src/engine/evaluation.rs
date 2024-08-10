use crate::prelude::*;

impl Engine {
    pub fn evaluate(&mut self) -> i32 {
        self.raw_evaluation() * self.board.active_colour.positive()
    }
    pub fn raw_evaluation(&mut self) -> i32 {
        let endgame = self.endgame();

        if !self.sufficient_material_to_force_checkmate() {
            return 0;
        }
        let mut total = 0;

        // punish kings next adjacent to open file
        for colour in [White, Black] {
            let file = self.board.piece_bitboards[colour + King].bitscan().file();
            let friendly_pawns = self.board.piece_bitboards[colour + Pawn];
            let enemy_pawns = self.board.piece_bitboards[!colour + Pawn];

            for pawns in [friendly_pawns, enemy_pawns] {
                let left_open = file.0 != 0 && pawns.filter_file(File(file.0 - 1)).count() == 0;
                let middle_open = pawns.filter_file(file).count() == 0;
                let right_open = file.0 != 7 && pawns.filter_file(File(file.0 + 1)).count() == 0;

                let num_open_files = left_open as i32 + middle_open as i32 + right_open as i32;
                total -= ((num_open_files * 35 * colour.positive()) as f32 * (1.0 - endgame)) as i32;
            }
        }

        // punish double pawns
        for file in 0..8 {
            let wp = self.board[WhitePawn].filter_file(File(file)).count() as i32;
            let bp = self.board[BlackPawn].filter_file(File(file)).count() as i32;

            total -= (wp - 1).max(0) * 20 * White.positive();
            total -= (bp - 1).max(0) * 20 * Black.positive();
        }
        // reward non-isolated pawns
        for colour in [White, Black] {
            let pawns = self.board[colour + Pawn];
            pawns.for_each(|pos| {
                let file = pos.file().0;

                let left_open = file == 0 || pawns.filter_file(File(file - 1)).count() == 0;
                let right_open = file == 7 || pawns.filter_file(File(file + 1)).count() == 0;

                if !(left_open && right_open) {
                    total += 15 * colour.positive();
                }
            });
        }
        // reward pawns close to king
        for colour in [White, Black] {
            let king = self.board[colour + King].bitscan();
            let pawns = self.board[colour + Pawn];
            pawns.for_each(|pos| {
                let is_adjacent = pos.file().0.abs_diff(king.file().0) <= 1;
                if !is_adjacent {
                    return;
                }
                let dif_rank = pos.rank().0.abs_diff(king.rank().0).saturating_sub(1);
                if dif_rank == 0 {
                    total += 15 * colour.positive();
                } else if dif_rank == 1 {
                    total += 10 * colour.positive();
                }
            });
        }
        // reward rooks on an open file
        for colour in [White, Black] {
            let friendly_pawns = self.board[colour + Pawn];
            let all_pawns = self.board.get(Pawn);
            self.board[colour + Rook].for_each(|pos| {
                if all_pawns.filter_file(pos.file()).count() == 0 {
                    total += 40 * colour.positive();
                } else if friendly_pawns.filter_file(pos.file()).count() == 0 {
                    total += 20 * colour.positive();
                }
            });
        }

        // reward bishop pair
        total += self.has_bishop_pair(White) as i32 * 20 * White.positive();
        total += self.has_bishop_pair(Black) as i32 * 20 * Black.positive();

        // square tables and piece values
        self.board[WhitePawn].for_each(|pos| total += piece_value_at_square(pos, WhitePawn, endgame));
        self.board[BlackPawn].for_each(|pos| total += piece_value_at_square(pos, BlackPawn, endgame));
        self.board[WhiteKnight].for_each(|pos| total += piece_value_at_square(pos, WhiteKnight, endgame));
        self.board[BlackKnight].for_each(|pos| total += piece_value_at_square(pos, BlackKnight, endgame));
        self.board[WhiteBishop].for_each(|pos| total += piece_value_at_square(pos, WhiteBishop, endgame));
        self.board[BlackBishop].for_each(|pos| total += piece_value_at_square(pos, BlackBishop, endgame));
        self.board[WhiteRook].for_each(|pos| total += piece_value_at_square(pos, WhiteRook, endgame));
        self.board[BlackRook].for_each(|pos| total += piece_value_at_square(pos, BlackRook, endgame));
        self.board[WhiteQueen].for_each(|pos| total += piece_value_at_square(pos, WhiteQueen, endgame));
        self.board[BlackQueen].for_each(|pos| total += piece_value_at_square(pos, BlackQueen, endgame));
        self.board[WhiteKing].for_each(|pos| total += piece_square_value(pos, WhiteKing, endgame));
        self.board[BlackKing].for_each(|pos| total += piece_square_value(pos, BlackKing, endgame));

        total
    }
    #[inline]
    #[must_use]
    pub fn has_bishop_pair(&self, side: Colour) -> bool {
        // Ignoring underpromotion for now
        self.board[side + Bishop].count() >= 2
    }
    #[inline]
    #[must_use]
    pub fn sufficient_material_to_force_checkmate(&self) -> bool {
        let w = self.board.side_bitboards(White);
        let b = self.board.side_bitboards(Black);

        w[Queen].count() > 0
            || b[Queen].count() > 0
            || w[Rook].count() > 0
            || b[Rook].count() > 0
            || w[Pawn].count() > 0
            || b[Pawn].count() > 0
            || self.has_bishop_pair(Colour::White)
            || self.has_bishop_pair(Colour::Black)
            || (w[Bishop].count() > 0 && w[Knight].count() > 0)
            || (b[Bishop].count() > 0 && b[Knight].count() > 0)
            || w[Knight].count() >= 3
            || b[Knight].count() >= 3
    }
}
#[inline]
#[must_use]
pub fn piece_value_at_square(pos: Pos, piece: Piece, endgame: f32) -> i32 {
    piece_value(piece, endgame) + piece_square_value(pos, piece, endgame)
}

#[inline]
#[must_use]
pub fn piece_value(piece: Piece, endgame: f32) -> i32 {
    abs_piece_value(piece.kind(), endgame) * piece.colour().positive()
}

#[inline]
#[must_use]
pub fn abs_piece_value_at_square(pos: Pos, piece: Piece, endgame: f32) -> i32 {
    abs_piece_value(piece.kind(), endgame) + abs_piece_square_value(pos, piece, endgame)
}

#[inline]
#[must_use]
pub fn abs_piece_value(piece: PieceKind, endgame: f32) -> i32 {
    let mg = [82, 337, 365, 477, 1025, 0][piece as usize];
    let eg = [94, 281, 297, 512, 936, 0][piece as usize];
    (mg as f32 * (1.0 - endgame) + eg as f32 * endgame) as i32
}

#[inline]
#[must_use]
pub fn piece_square_value(pos: Pos, piece: Piece, endgame: f32) -> i32 {
    abs_piece_square_value(pos, piece, endgame) * piece.colour().positive()
}

#[inline]
#[must_use]
pub fn abs_piece_square_value(pos: Pos, piece: Piece, endgame: f32) -> i32 {
    let index = if piece.is_white() { Pos::new(Rank(7 - pos.rank().0), pos.file()) } else { pos };
    let mg = square_tables::MG[piece.kind() as usize][index];
    let eg = square_tables::EG[piece.kind() as usize][index];

    (mg as f32 * (1.0 - endgame) + eg as f32 * endgame) as i32
}

#[rustfmt::skip]
mod square_tables {
    pub const MG: [[i32; 64]; 6] = [MG_PAWN, MG_KNIGHT, MG_BISHOP, MG_ROOK, MG_QUEEN, MG_KING];
    pub const EG: [[i32; 64]; 6] = [EG_PAWN, EG_KNIGHT, EG_BISHOP, EG_ROOK, EG_QUEEN, EG_KING];

    pub const MG_PAWN: [i32; 64] = [
        0,  0,  0,  0,  0,  0, 0,  0,
       98,134, 61, 95, 68,126,34,-11,
       -6,  7, 26, 31, 65, 56,25,-20,
      -14, 13,  6, 21, 23, 12,17,-23,
      -27, -2, -5, 12, 17,  6,10,-25,
      -26, -4, -4,-10,  3,  3,33,-12,
      -35, -1,-20,-23,-15, 24,38,-22,
        0,  0,  0,  0,  0,  0, 0,  0,
  ];
  
  pub const EG_PAWN: [i32; 64] = [
        0,  0,  0,  0,  0,  0,  0,  0,
      178,173,158,134,147,132,165,187,
       94,100, 85, 67, 56, 53, 82, 84,
       32, 24, 13,  5, -2,  4, 17, 17,
       13,  9, -3, -7, -7, -8,  3, -1,
        4,  7, -6,  1,  0, -5, -1, -8,
       13,  8,  8, 10, 13,  0,  2, -7,
        0,  0,  0,  0,  0,  0,  0,  0,
  ];
  
  pub const MG_KNIGHT: [i32; 64] = [
      -167,-89,-34,-49, 61,-97,-15,-107,
       -73,-41, 72, 36, 23, 62,  7, -17,
       -47, 60, 37, 65, 84,129, 73,  44,
        -9, 17, 19, 53, 37, 69, 18,  22,
       -13,  4, 16, 13, 28, 19, 21,  -8,
       -23, -9, 12, 10, 19, 17, 25, -16,
       -29,-53,-12, -3, -1, 18,-14, -19,
      -105,-21,-58,-33,-17,-28,-19, -23,
  ];
  
  pub const EG_KNIGHT: [i32; 64] = [
      -58,-38,-13,-28,-31,-27,-63,-99,
      -25, -8,-25, -2, -9,-25,-24,-52,
      -24,-20, 10,  9, -1, -9,-19,-41,
      -17,  3, 22, 22, 22, 11,  8,-18,
      -18, -6, 16, 25, 16, 17,  4,-18,
      -23, -3, -1, 15, 10, -3,-20,-22,
      -42,-20,-10, -5, -2,-20,-23,-44,
      -29,-51,-23,-15,-22,-18,-50,-64,
  ];
  
  pub const MG_BISHOP: [i32; 64] = [
      -29,  4,-82,-37,-25,-42,  7, -8,
      -26, 16,-18,-13, 30, 59, 18,-47,
      -16, 37, 43, 40, 35, 50, 37, -2,
       -4,  5, 19, 50, 37, 37,  7, -2,
       -6, 13, 13, 26, 34, 12, 10,  4,
        0, 15, 15, 15, 14, 27, 18, 10,
        4, 15, 16,  0,  7, 21, 33,  1,
      -33, -3,-14,-21,-13,-12,-39,-21,
  ];
  
  pub const EG_BISHOP: [i32; 64] = [
      -14,-21,-11, -8,-7, -9,-17,-24,
       -8, -4,  7,-12,-3,-13, -4,-14,
        2, -8,  0, -1,-2,  6,  0,  4,
       -3,  9, 12,  9,14, 10,  3,  2,
       -6,  3, 13, 19, 7, 10, -3, -9,
      -12, -3,  8, 10,13,  3, -7,-15,
      -14,-18, -7, -1, 4, -9,-15,-27,
      -23, -9,-23, -5,-9,-16, -5,-17,
  ];
  
  pub const MG_ROOK: [i32; 64] = [
       32, 42, 32, 51,63, 9, 31, 43,
       27, 32, 58, 62,80,67, 26, 44,
       -5, 19, 26, 36,17,45, 61, 16,
      -24,-11,  7, 26,24,35, -8,-20,
      -36,-26,-12, -1, 9,-7,  6,-23,
      -45,-25,-16,-17, 3, 0, -5,-33,
      -44,-16,-20, -9,-1,11, -6,-71,
      -19,-13,  1, 17,16, 7,-37,-26,
  ];
  
  pub const EG_ROOK: [i32; 64] = [
      13,10,18,15,12, 12,  8,  5,
      11,13,13,11,-3,  3,  8,  3,
       7, 7, 7, 5, 4, -3, -5, -3,
       4, 3,13, 1, 2,  1, -1,  2,
       3, 5, 8, 4,-5, -6, -8,-11,
      -4, 0,-5,-1,-7,-12, -8,-16,
      -6,-6, 0, 2,-9, -9,-11, -3,
      -9, 2, 3,-1,-5,-13,  4,-20,
  ];
  
  pub const MG_QUEEN: [i32; 64] = [
      -28,  0, 29, 12, 59, 44, 43, 45,
      -24,-39, -5,  1,-16, 57, 28, 54,
      -13,-17,  7,  8, 29, 56, 47, 57,
      -27,-27,-16,-16, -1, 17, -2,  1,
       -9,-26, -9,-10, -2, -4,  3, -3,
      -14,  2,-11, -2, -5,  2, 14,  5,
      -35, -8, 11,  2,  8, 15, -3,  1,
       -1,-18, -9, 10,-15,-25,-31,-50,
  ];
  
  pub const EG_QUEEN: [i32; 64] = [
       -9, 22, 22, 27, 27, 19, 10, 20,
      -17, 20, 32, 41, 58, 25, 30,  0,
      -20,  6,  9, 49, 47, 35, 19,  9,
        3, 22, 24, 45, 57, 40, 57, 36,
      -18, 28, 19, 47, 31, 34, 39, 23,
      -16,-27, 15,  6,  9, 17, 10,  5,
      -22,-23,-30,-16,-16,-23,-36,-32,
      -33,-28,-22,-43, -5,-32,-20,-41,
  ];
  
  pub const MG_KING: [i32; 64] = [
      -65, 23, 16,-15,-56,-34,  2, 13,
       29, -1,-20, -7, -8, -4,-38,-29,
       -9, 24,  2,-16,-20,  6, 22,-22,
      -17,-20,-12,-27,-30,-25,-14,-36,
      -49, -1,-27,-39,-46,-44,-33,-51,
      -14,-14,-22,-46,-44,-30,-15,-27,
        1,  7, -8,-64,-43,-16,  9,  8,
      -15, 36, 12,-54,  8,-28, 24, 14,
  ];
  
  pub const EG_KING: [i32; 64] = [
      -74,-35,-18,-18,-11, 15,  4,-17,
      -12, 17, 14, 17, 17, 38, 23, 11,
       10, 17, 23, 15, 20, 45, 44, 13,
       -8, 22, 24, 27, 26, 33, 26,  3,
      -18, -4, 21, 24, 27, 23,  9,-11,
      -19, -3, 11, 21, 23, 16,  7, -9,
      -27,-11,  4, 13, 14,  4, -5,-17,
      -53,-34,-21,-11,-28,-14,-24,-43
  ];
}
