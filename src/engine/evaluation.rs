use super::mobility::raw_mobility_eval;
use crate::{core::board::Pieces, prelude::*};

const ROOK_SAME_FILE_BONUS: i32 = 20;

#[must_use]
pub fn raw_evaluation(board: &Board) -> i32 {
    let phase = phase(board);
    if !board.sufficient_material() {
        return 0;
    }
    let mut final_total = 0;

    for side in [White, Black] {
        let [mut total, mut earlygame, endgame] = [0; 3];
        let king = board.get_king_square(side).unwrap();
        let friendly = board.side_bitboards(side);
        let enemy = board.side_bitboards(!side);

        earlygame += king_safety(board, side, &enemy, king);
        earlygame += punish_open_kings(king, &friendly, &enemy);
        total += punish_double_pawns(&friendly);
        total += reward_pawns_close_to_king(&friendly, king);
        total += reward_outposts(side, &friendly, &enemy);
        total += reward_rooks_on_open_file(&friendly, board);
        total += reward_lined_up_rooks(&friendly, board);
        total += has_bishop_pair(board, side) as i32 * 50;

        // punish pieces in front of enemy pawns
        let pawn_attacks = board.pawn_attacks(!side);

        let non_pawns = friendly[Knight] | friendly[Bishop] | friendly[Rook] | friendly[Queen];
        total -= (pawn_attacks & non_pawns).count() as i32 * 40;

        for sq in friendly[Pawn] {
            // reward non-isolated pawns
            if !(sq.file().adjacency_mask() & friendly[Pawn]).is_empty() {
                total += [15, 18, 23, 25, 25, 23, 18, 15][sq.file()];
            }
            // reward passed pawns
            let is_passed_pawn = (sq.passed_pawn_mask(side) & enemy[Pawn]).is_empty();
            if is_passed_pawn {
                let offset = sq.rank().relative_to(side);
                total += [0, 10, 20, 30, 40, 50, 70, 90][offset];
            }
        }

        total += earlygame * phase.earlygame() + endgame * phase.endgame();
        final_total += total * side.positive();
    }
    let psqt = board.mg_psqt * phase.earlygame() + board.eg_psqt * phase.endgame();
    final_total += psqt;

    // mop up evaluation
    let mop_up_side = match final_total {
        100.. => Some(White),
        ..=-100 => Some(Black),
        _ => None,
    };
    if let Some(mop_up_side) = mop_up_side {
        let md = board.active_king().unwrap().manhattan_distance(board.inactive_king().unwrap());
        let cmd = board.get_king_square(!mop_up_side).unwrap().centre_manhattan_distance() as i32;
        let mop_up_score = (47 * cmd + 16 * (14 - md as i32)) * mop_up_side.positive();
        final_total += mop_up_score * phase.endgame();
    }
    final_total + raw_mobility_eval(board) + psqt
}

fn king_safety(board: &Board, side: Side, enemy: &Pieces, king: Square) -> i32 {
    let mut total = 0;
    total -= (enemy[Queen] & king.nearby3()).count() as i32 * 30;
    total -= (enemy[Knight] | enemy[Bishop] & king.nearby3()).count() as i32 * 15;
    total -= (enemy[Rook] & king.nearby3()).count() as i32 * 20;

    total -= (enemy[Queen] & king.nearby2()).count() as i32 * 30;
    total -= (enemy[Knight] | enemy[Bishop] & king.nearby2()).count() as i32 * 15;
    total -= (enemy[Rook] | enemy[Rook] & king.nearby3()).count() as i32 * 20;

    total -= i32::from(queen_attacks(king, board.all_pieces() & !board[side]).count()) * 5;
    total
}

fn punish_open_kings(king: Square, friendly: &Pieces, enemy: &Pieces) -> i32 {
    let mut total = 0;
    for pawns in [friendly[Pawn], enemy[Pawn]] {
        let file = king.file();

        let left_open = file.sub_int(1).is_some_and(|file| (pawns & file.mask()).is_empty());
        let middle_open = (pawns & file.mask()).is_empty();
        let right_open = file.add_int(1).is_some_and(|file| (pawns & file.mask()).is_empty());

        let num_open_files = left_open as i32 + middle_open as i32 + right_open as i32;

        total -= num_open_files * [40, 35, 25, 10, 10, 25, 35, 40][file];
    }
    total
}

fn punish_double_pawns(friendly: &Pieces) -> i32 {
    let mut total = 0;
    for file in File::ALL {
        let pawns_in_file = (friendly[Pawn] & file.mask()).count();
        total -= i32::from(pawns_in_file.saturating_sub(1)) * 25;
    }
    total
}

fn reward_pawns_close_to_king(friendly: &Pieces, king: Square) -> i32 {
    let mut total = 0;
    for sq in friendly[Pawn] & king.nearby2() & (king.file().adjacency_mask() | king.file().mask())
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
        let dif_rank = sq.rank().diff(king.rank());
        unsafe { std::hint::assert_unchecked(dif_rank < 3) };
        total += BONUSES[sq.file()][dif_rank as usize];
    }
    total
}

fn reward_outposts(side: Side, friendly: &Pieces, enemy: &Pieces) -> i32 {
    let mut total = 0;
    for sq in Bitboard::ALL.shift_forward_n(side, 4) & (friendly[Knight] | friendly[Bishop]) {
        let is_outpost = (sq.outpost_mask(side) & enemy[Pawn]).is_empty();
        if is_outpost {
            total += 20;
        }
    }

    total
}

fn reward_rooks_on_open_file(friendly: &Pieces, board: &Board) -> i32 {
    let mut total = 0;
    for sq in friendly[Rook] {
        if (board[Pawn] & sq.file().mask()).is_empty() {
            total += 20;
        } else if (friendly[Pawn] & (sq.file().mask())).is_empty() {
            total += 10;
        }
    }
    total
}

fn reward_lined_up_rooks(friendly: &Pieces, board: &Board) -> i32 {
    let mut total = 0;
    if friendly[Rook].count() >= 2 {
        let rook_a = unsafe { friendly[Rook].bitscan_unchecked() };
        let rook_b = unsafe { friendly[Rook].rbitscan_unchecked() };

        let rook_attacks = rook_attacks(rook_a, board.all_pieces());
        if rook_attacks.contains(rook_b) {
            total += 20;
            total += (rook_a.file() == rook_b.file()) as i32 * ROOK_SAME_FILE_BONUS;
        }
    }
    total
}

fn has_bishop_pair(board: &Board, side: Side) -> bool {
    // Ignoring underpromotion for now
    board.get(side + Bishop).count() >= 2
}

#[must_use]
pub fn evaluate(board: &Board) -> Score {
    Score(raw_evaluation(board) * board.active_side.positive())
}

#[test]
fn sanity_test() {
    assert!(raw_evaluation(&Board::start_pos()) >= 0);
}
