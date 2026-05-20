mod mobility;

use crate::prelude::*;

#[must_use]
pub fn evaluation(board: &Board) -> Score {
    debug_evaluation(board, |_, _, _| {})
}

pub fn debug_evaluation(board: &Board, debug: impl FnMut(&'static str, Side, i32)) -> Score {
    Score(Evaluation::new(board, debug).evaluate())
}

struct Evaluation<'a, F> {
    both: i32,
    total: [i32; 2],
    earlygame: [i32; 2],
    endgame: [i32; 2],
    board: &'a Board,
    king: [Square; 2],
    pawn_attacks: [Bitboard; 2],
    occupancy: Bitboard,
    debug: F,
}

impl<'a, F: FnMut(&'static str, Side, i32)> Evaluation<'a, F> {
    pub fn new(board: &'a Board, debug: F) -> Self {
        Self {
            both: 0,
            total: [0; 2],
            earlygame: [0; 2],
            endgame: [0; 2],
            king: [board.get_king_square(Black).unwrap(), board.get_king_square(White).unwrap()],
            occupancy: board.all_pieces(),
            board,
            pawn_attacks: [board.pawn_attacks(Black), board.pawn_attacks(White)],
            debug,
        }
    }

    pub fn evaluate(&mut self) -> i32 {
        if !self.board.sufficient_material() {
            return 0;
        }
        let phase = phase(self.board);

        for side in [White, Black] {
            macro_rules! heuristics {
                ($($name:ident),* $(,)?) => {
                    $(
                        let total = self.total[side]
                            + self.earlygame[side] * phase.earlygame()
                            + self.endgame[side] * phase.endgame();

                        let _: () = self.$name(side);

                        let new_total = self.total[side]
                            + self.earlygame[side] * phase.earlygame()
                            + self.endgame[side] * phase.endgame();

                        if total != new_total {
                            (self.debug)(stringify!($name), side, new_total - total);
                        }
                    );*

                };
            }
            heuristics!(
                king_safety,
                open_kings,
                double_pawns,
                pawns_close_to_king,
                // outposts,
                rooks_on_open_file,
                lined_up_rooks,
                bishop_pair,
                passed_pawns,
                isolated_pawns,
                attacked_by_pawn,
                mobility,
            );
        }
        self.castling_rights();
        self.both += self.board.active_side.positive() * 5;
        self.both += self.board.mg_psqt * phase.earlygame() + self.board.eg_psqt * phase.endgame();

        let mut total = self.both + self.total[White] - self.total[Black];
        total += (self.earlygame[White] - self.earlygame[Black]) * phase.earlygame();
        total += (self.endgame[White] - self.endgame[Black]) * phase.endgame();

        self.mop_up_evaluation(phase, &mut total);

        total
    }

    fn king_safety(&mut self, side: Side) {
        let nearby3 = self.king[side].nearby3();
        let nearby2 = self.king[side].nearby2();

        self.earlygame[side] -= (self.board.get(!side + Bishop) & nearby2).count() as i32 * 15;
        self.earlygame[side] -= (self.board.get(!side + Knight) & nearby2).count() as i32 * 15;
        self.earlygame[side] -= (self.board.get(!side + Queen) & nearby2).count() as i32 * 30;

        self.earlygame[side] -= (self.board.get(!side + Bishop) & nearby3).count() as i32 * 15;
        self.earlygame[side] -= (self.board.get(!side + Knight) & nearby3).count() as i32 * 15;
        self.earlygame[side] -= (self.board.get(!side + Rook) & nearby3).count() as i32 * 40;
        self.earlygame[side] -= (self.board.get(!side + Queen) & nearby3).count() as i32 * 30;

        self.earlygame[side] -=
            i32::from((queen_attacks(self.king[side], self.occupancy) & !self.board[side]).count())
                * 10;
    }

    fn open_kings(&mut self, side: Side) {
        for pawns in [self.board.get(side + Pawn), self.board.get(!side + Pawn)] {
            let file = self.king[side].file();

            let left_open = file.sub_int(1).is_some_and(|file| (pawns & file.mask()).is_empty());
            let middle_open = (pawns & file.mask()).is_empty();
            let right_open = file.add_int(1).is_some_and(|file| (pawns & file.mask()).is_empty());

            let num_open_files = left_open as i32 + middle_open as i32 + right_open as i32;

            self.earlygame[side] -= num_open_files * [40, 35, 25, 10, 10, 25, 35, 40][file];
        }
    }

    fn double_pawns(&mut self, side: Side) {
        for file in File::ALL {
            let pawns_in_file = (self.board.get(side + Pawn) & file.mask()).count();
            self.total[side] -= i32::from(pawns_in_file.saturating_sub(1)) * 25;
        }
    }

    fn pawns_close_to_king(&mut self, side: Side) {
        for sq in self.board.get(side + Pawn)
            & self.king[side].nearby2()
            & (self.king[side].file().adjacency_mask() | self.king[side].file().mask())
        {
            const BONUSES: [[i32; 3]; 8] = [
                [18, 18, 14],
                [15, 15, 10],
                [13, 13, 9],
                [8, 8, 4],
                [8, 8, 4],
                [13, 13, 9],
                [15, 15, 10],
                [18, 18, 14],
            ];
            let dif_rank = sq.rank().diff(self.king[side].rank());
            unsafe { std::hint::assert_unchecked(dif_rank < 3) };
            self.earlygame[side] += BONUSES[sq.file()][dif_rank as usize];
        }
    }

    #[expect(unused)]
    fn outposts(&mut self, side: Side) {
        for sq in Bitboard::ALL.shift_forward_n(side, 4)
            & (self.board.get(side + Knight) | self.board.get(side + Bishop))
        {
            let is_outpost = (sq.outpost_mask(side) & self.board.get(!side + Pawn)).is_empty();
            if is_outpost {
                self.total[side] += 20;
            }
        }
    }

    fn rooks_on_open_file(&mut self, side: Side) {
        for sq in self.board.get(side + Rook) {
            if (self.board[Pawn] & sq.file().mask()).is_empty() {
                self.total[side] += 20;
            } else if (self.board.get(side + Pawn) & (sq.file().mask())).is_empty() {
                self.total[side] += 10;
            }
        }
    }

    fn lined_up_rooks(&mut self, side: Side) {
        if self.board.get(side + Rook).count() >= 2 {
            let rook_a = unsafe { self.board.get(side + Rook).bitscan_unchecked() };
            let rook_b = unsafe { self.board.get(side + Rook).rbitscan_unchecked() };

            let rook_attacks = rook_attacks(rook_a, self.occupancy);
            if rook_attacks.contains(rook_b) {
                self.total[side] += 20;
                self.total[side] += (rook_a.file() == rook_b.file()) as i32 * 20;
            }
        }
    }

    fn castling_rights(&mut self) {
        self.both += self.board.can_castle.intersects(CanCastle::BOTH_WHITE) as i32 * 20;
        self.both -= self.board.can_castle.intersects(CanCastle::BOTH_BLACK) as i32 * 20;
    }

    fn bishop_pair(&mut self, side: Side) {
        // Ignoring underpromotion for now
        self.total[side] += (self.board.get(side + Bishop).count() >= 2) as i32 * 50;
    }

    fn attacked_by_pawn(&mut self, side: Side) {
        let pawn_attacks = self.pawn_attacks[!side];

        let non_pawns = self.board.get(side + Knight)
            | self.board.get(side + Bishop)
            | self.board.get(side + Rook)
            | self.board.get(side + Queen);
        self.total[side] -= (pawn_attacks & non_pawns).count() as i32 * 40;
    }

    fn passed_pawns(&mut self, side: Side) {
        for sq in self.board.get(side + Pawn) {
            if (sq.passed_pawn_mask(side) & self.board.get(!side + Pawn)).is_empty() {
                let offset = sq.rank().relative_to(side);
                self.total[side] += [0, 10, 20, 30, 40, 50, 70, 90][offset];
            }
        }
    }

    fn isolated_pawns(&mut self, side: Side) {
        for sq in self.board.get(side + Pawn) {
            if (sq.file().adjacency_mask() & self.board.get(side + Pawn)).is_empty() {
                self.total[side] -= [15, 18, 23, 25, 25, 23, 18, 15][sq.file()];
            }
        }
    }

    fn mop_up_evaluation(&mut self, phase: Phase, total: &mut i32) {
        let mop_up_side = match *total {
            100.. => Some(White),
            ..=-100 => Some(Black),
            _ => None,
        };
        if let Some(mop_up_side) = mop_up_side {
            let md = self
                .board
                .active_king()
                .unwrap()
                .manhattan_distance(self.board.inactive_king().unwrap());
            let cmd = self.board.get_king_square(!mop_up_side).unwrap().centre_manhattan_distance()
                as i32;
            let mop_up_score = (47 * cmd + 16 * (14 - md as i32)) * mop_up_side.positive();
            *total += mop_up_score * phase.endgame();
        }
    }
}

#[test]
fn sanity_test() {
    assert!(evaluation(&Board::start_pos()).0 >= 0);
}
