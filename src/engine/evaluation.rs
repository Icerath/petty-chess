use crate::prelude::*;

const ROOK_SAME_FILE_BONUS: i32 = 20;

impl Engine {
    pub fn evaluate(&mut self) -> i32 {
        self.raw_evaluation() * self.board.active_side.positive()
    }
    #[allow(clippy::too_many_lines)]
    pub fn raw_evaluation(&mut self) -> i32 {
        let phase = self.phase();

        if !self.sufficient_material_to_force_checkmate() {
            return 0;
        }
        let mut final_total = 0;

        for side in [White, Black] {
            let mut total = 0;
            let Some(king) = self.board.get_king_square(side) else { continue };
            let friendly = self.board.side_bitboards(side);
            let enemy = self.board.side_bitboards(!side);
            // punish kings next adjacent to open file
            for pawns in [friendly[Pawn], enemy[Pawn]] {
                const PENALTIES: [i32; 8] = [40, 35, 25, 10, 10, 25, 35, 40];
                let file = king.file();
                let penalty = PENALTIES[file.0 as usize];

                let left_open = file.0 != 0 && (pawns & File(file.0 - 1).mask()).is_empty();
                let middle_open = (pawns & file.mask()).is_empty();
                let right_open = file.0 != 7 && (pawns & (File(file.0 + 1).mask())).is_empty();

                let num_open_files = left_open as i32 + middle_open as i32 + right_open as i32;
                total -= (num_open_files * penalty) * phase.earlygame();
            }
            // punish double pawns
            for file in 0..8 {
                let pawns_in_file = (friendly[Pawn] & (File(file).mask())).count() as i32;
                total -= (pawns_in_file - 1).max(0) * 25;
            }
            // reward non-isolated pawns
            friendly[Pawn].for_each(|sq| {
                let file = sq.file();
                let left_open = (friendly[Pawn] & (file - 1).mask()).is_empty();
                let right_open = (friendly[Pawn] & (file + 1).mask()).is_empty();

                if !(left_open && right_open) {
                    let distance = file.0.abs_diff(4).min(file.0.abs_diff(3));
                    total += match distance {
                        0 => 25,
                        1 => 23,
                        2 => 18,
                        3 => 15,
                        _ => unreachable!(),
                    };
                }
            });
            // reward passed pawns
            friendly[Pawn].for_each(|sq| {
                const BONUSES: [i32; 8] = [0, 10, 20, 30, 40, 50, 70, 90];
                let is_passed_pawn = (sq.passed_pawn_mask(side) & enemy[Pawn]).is_empty();
                if is_passed_pawn {
                    let offset = match side {
                        Side::White => sq.rank().0 as usize,
                        Side::Black => 7 - sq.rank().0 as usize,
                    };
                    total += BONUSES[offset];
                }
            });
            // reward outposts
            (friendly[Knight] | friendly[Bishop]).for_each(|sq| {
                if sq.rank().relative_to(side).0 < 4 {
                    return;
                }
                let is_outpost = (sq.outpost_mask(side) & enemy[Pawn]).is_empty();
                if is_outpost {
                    total += 20;
                }
            });
            // reward pawns close to king
            let kadj_pawns_mask = (king.file() - 1).mask() | (king.file() + 1).mask() | king.file().mask();
            (friendly[Pawn] & kadj_pawns_mask).for_each(|sq| {
                const BONUSES: [[i32; 2]; 8] =
                    [[18, 14], [15, 10], [13, 9], [8, 4], [8, 4], [13, 9], [15, 10], [18, 14]];

                let dif_rank = sq.rank().0.abs_diff(king.rank().0).saturating_sub(1);
                total += BONUSES[sq.file().0 as usize].get(dif_rank as usize).unwrap_or(&0);
            });
            // reward rooks on an open file
            friendly[Rook].for_each(|sq| {
                if (self.board[Pawn] & sq.file().mask()).is_empty() {
                    total += 20;
                } else if (friendly[Pawn] & (sq.file().mask())).is_empty() {
                    total += 10;
                }
            });
            // reward rooks able to see eachother
            if let (Some(rook_a), Some(rook_b)) = (friendly[Rook].bitscan(), friendly[Rook].rbitscan()) {
                let rook_attacks = self.magic.rook_attacks(rook_a, self.board.all_pieces());
                if rook_attacks.contains(rook_b) {
                    total += 20 + (rook_a.file() == rook_b.file()) as i32 * ROOK_SAME_FILE_BONUS;
                }
            }
            // reward bishop pair
            total += self.has_bishop_pair(side) as i32 * 50;
            // material and piece square table values
            for piecekind in [Pawn, Knight, Bishop, Rook, Queen] {
                let piece = side + piecekind;
                self.board
                    .get(piece)
                    .for_each(|square| total += abs_piece_value_at_square(square, piece, phase));
            }
            total += abs_piece_square_value(king, side + King, phase);

            final_total += total * side.positive();
        }
        // mop up evaluation
        let mop_up_side = match final_total {
            100.. => Some(White),
            ..=-100 => Some(Black),
            _ => None,
        };
        if let Some(mop_up_side) = mop_up_side {
            if let (Some(active_king), Some(inactive_king)) =
                (self.board.active_king(), self.board.inactive_king())
            {
                let md = active_king.manhattan_distance(inactive_king);
                let cmd = self.board.get_king_square(!mop_up_side).unwrap().centre_manhattan_distance() as i32;
                let mop_up_score = (47 * cmd + 16 * (14 - md as i32)) * mop_up_side.positive();
                final_total += mop_up_score * phase.endgame();
            }
        }
        let mobility_score = self.raw_mobility_eval();
        final_total + mobility_score
    }
    #[inline]
    fn has_bishop_pair(&self, side: Side) -> bool {
        // Ignoring underpromotion for now
        self.board.get(side + Bishop).count() >= 2
    }
    #[inline]
    #[must_use]
    pub fn sufficient_material_to_force_checkmate(&self) -> bool {
        let w = self.board.side_bitboards(White);
        let b = self.board.side_bitboards(Black);

        !w[Queen].is_empty()
            || !b[Queen].is_empty()
            || !w[Rook].is_empty()
            || !b[Rook].is_empty()
            || !w[Pawn].is_empty()
            || !b[Pawn].is_empty()
            || self.has_bishop_pair(Side::White)
            || self.has_bishop_pair(Side::Black)
            || (!w[Bishop].is_empty() && !w[Knight].is_empty())
            || (!b[Bishop].is_empty() && !b[Knight].is_empty())
            || w[Knight].0 >= 3
            || b[Knight].0 >= 3
    }
}
#[inline]
#[must_use]
pub fn piece_value_at_square(sq: Square, piece: Piece, phase: Phase) -> i32 {
    piece_value(piece, phase) + piece_square_value(sq, piece, phase)
}

#[inline]
#[must_use]
pub fn piece_value(piece: Piece, phase: Phase) -> i32 {
    abs_piece_value(piece.kind(), phase) * piece.side().positive()
}

#[inline]
#[must_use]
pub fn abs_piece_value_at_square(sq: Square, piece: Piece, phase: Phase) -> i32 {
    abs_piece_value(piece.kind(), phase) + abs_piece_square_value(sq, piece, phase)
}

#[inline]
#[must_use]
pub fn abs_piece_value(piece: PieceKind, phase: Phase) -> i32 {
    let mg = [82, 337, 365, 477, 1025, 0][piece as usize];
    let eg = [94, 281, 297, 512, 936, 0][piece as usize];
    mg * phase.earlygame() + eg * phase.endgame()
}

#[inline]
#[must_use]
pub fn piece_square_value(sq: Square, piece: Piece, phase: Phase) -> i32 {
    abs_piece_square_value(sq, piece, phase) * piece.side().positive()
}

#[inline]
#[must_use]
pub fn abs_piece_square_value(sq: Square, piece: Piece, phase: Phase) -> i32 {
    let index = if piece.is_white() { sq.flip() } else { sq };
    let mg = square_tables::MG[piece.kind() as usize][index];
    let eg = square_tables::EG[piece.kind() as usize][index];

    mg * phase.earlygame() + eg * phase.endgame()
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
      -83,-44,-17,-24, 30,-48,-7,-53,
      -36,-20, 36, 18, 11, 31,  3, -8,
      -23, 30, 18, 32, 42, 64, 36, 22,
       -4,  8,  9, 26, 18, 34,  9, 11,
       -6,  2,  8,  6, 14,  9, 10, -4,
      -11, -4,  6,  5,  9,  8, 12, -8,
      -14,-26, -6, -1, -0,  9, -7, -9,
      -52,-10,-29,-16, -8,-14, -9,-11,
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
