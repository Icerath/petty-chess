use super::{
    Engine, evaluation::evaluation, movelist::MoveList, phase::phase, transposition::Nodetype,
};
use crate::{
    engine::score::Score,
    prelude::*,
    uci::{Info, UciResponse},
};

impl Engine {
    pub fn search_root(&mut self) {
        self.total_nodes = 0;

        self.killer.clear();
        self.killer.extend([None; 64]);

        let mut best_move = *self.board.legal_moves().first().unwrap_or(&Move::NULL);

        for depth in 1..=255 {
            let mut new_pv = Moves::new();
            let Ok(score) = self.search(-Score::MAX, Score::MAX, depth, &mut new_pv) else {
                break;
            };
            self.total_nodes -= 1;
            self.pv = new_pv.iter().copied().rev().collect();
            best_move = *self.pv.first().unwrap_or(&best_move);

            let time_taken = self.time_started.elapsed();
            let info = Info {
                depth: Some(depth as u32),
                score: Some(score),
                nodes: Some(self.total_nodes),
                time: Some(time_taken),
                nps: Some((self.total_nodes as f64 / time_taken.as_secs_f64()) as u32),
                pv: Some(self.pv.clone()),
                seldepth: Some(self.seldepth.into()),
                ..Info::default()
            };
            println!("{}", UciResponse::Info(Box::new(info)));

            if let Some(mate) = score.mate_ply()
                && mate <= i16::from(depth)
            {
                break;
            }

            let skip_time = 2.2 - phase(&self.board).endgame().as_float();
            if self.time_started.elapsed().mul_f32(skip_time) > self.time_available {
                break;
            }
        }
        println!("{}", UciResponse::Bestmove { mov: best_move, ponder: None });
    }

    fn seen_position(&self) -> bool {
        self.seen_positions.iter().filter(|&&sq| sq == self.board.zobrist).count() > 1
    }

    #[expect(clippy::too_many_lines)]
    pub(crate) fn search(
        &mut self,
        mut alpha: Score,
        beta: Score,
        depth: u8,
        pline: &mut Moves,
    ) -> Result<Score, ()> {
        if self.is_cancelled() {
            return Err(());
        }

        if self.depth_from_root != 0 && self.seen_position() {
            return Ok(if self.depth_from_root.is_multiple_of(2) { -Score(20) } else { Score(20) });
        }
        if self.depth_from_root != 0 && self.board.halfmove_clock >= 50 {
            return Ok(Score(0));
        }

        if depth == 0 {
            return Ok(self.search_captures(alpha, beta));
        }

        // the wrapping_sub here seems questionable, but replacing it with overflow aware code seems to reduce ELO
        if depth != 1 && beta.0.wrapping_sub(alpha.0) > 1 {
            let score = self.search(Score(beta.0 - 1), beta, depth, &mut Moves::new())?;
            if score >= beta {
                return Ok(score);
            }
        }

        let mut tt_move = None;
        if self.depth_from_root > 0
            && let Some(entry) = self.transposition_table.get(self.board.zobrist)
        {
            tt_move = entry.mov.opt();
            if let Some(score) = entry.score(alpha, beta, depth) {
                if let Some(tt_move) = tt_move {
                    pline.push(tt_move);
                }
                return Ok(score);
            }
        }

        if self.should_null_move_heuristic(depth) {
            let unmake = self.board.make_null_move();
            self.depth_from_root += 1;
            let score = -self.search(-beta, -(Score(beta.0 - 1)), depth - 3, &mut Moves::new())?;
            self.depth_from_root -= 1;
            self.board.unmake_null_move(unmake);

            if score >= beta {
                return Ok(score);
            }
        }

        let mut moves = MoveList::default();
        let alpha_orig = alpha;

        let mut best_score = -Score::MAX;
        let mut best_move = None;
        let mut move_count = 0;
        let killer = self.killer[self.depth_from_root as usize];

        let no_tt_move = tt_move.is_none();

        if no_tt_move && depth > 8 {
            let mut line = Moves::new();
            self.search(alpha, beta, depth / 2, &mut line)?;
            if let Some(best) = line.pop() {
                tt_move = Some(best);
            }
        }

        while let Some(mov) = moves.next::<false>(self, tt_move, killer) {
            if !self.board.is_legal(mov) {
                continue;
            }
            move_count += 1;

            let unmake = self.board.make_move(mov);
            self.seen_positions.push(self.board.zobrist);

            let mut next_depth = depth - 1;

            // extentions and reductions
            let checking = self.board.in_check();
            if checking {
                next_depth += 1;
            }
            let mut late_move_reduction = 0;
            if depth > 2 && move_count > 1 {
                late_move_reduction += 1;
                if !mov.flags().is_capture() && !checking && move_count > 3 {
                    late_move_reduction += 1;
                }
            }
            next_depth -= late_move_reduction;

            if depth > 5 && no_tt_move {
                if move_count > 1 {
                    next_depth -= 1;
                }
                if tt_move.is_none() {
                    next_depth -= 1;
                }
            }

            self.depth_from_root += 1;
            let mut line = Moves::new();
            let mut score = -self.search(-beta, -alpha, next_depth, &mut line)?;

            if score >= beta && late_move_reduction > 0 {
                line.clear();
                score = -self.search(-beta, -alpha, next_depth + late_move_reduction, &mut line)?;
            } else if score > alpha && late_move_reduction > 1 {
                line.clear();
                score = -self.search(-beta, -alpha, next_depth + 1, &mut line)?;
            }

            self.depth_from_root -= 1;

            self.seen_positions.pop();
            self.board.unmake_move(unmake);

            if score > best_score {
                line.push(mov);
                *pline = line;
                best_score = score;
                best_move = Some(mov);
            }
            alpha = alpha.max(best_score);

            if score >= beta {
                if !mov.flags().is_capture() && mov.flags().promotion().is_none() {
                    self.killer[self.depth_from_root as usize] = Some(mov);
                    let piece = self.board.get_square(mov.from()).unwrap();
                    self.history_table[piece][mov.to()] += u32::from(depth) * u32::from(depth);
                }
                break;
            }
        }

        if move_count == 0 {
            return Ok(if self.board.in_check() {
                -Score::mate_in_ply(self.depth_from_root)
            } else {
                Score(0)
            });
        }
        let nodetype = if best_score <= alpha_orig {
            Nodetype::Alpha
        } else if best_score >= beta {
            Nodetype::Beta
        } else {
            Nodetype::Exact
        };
        self.transposition_table.insert(self.board.zobrist, depth, best_score, nodetype, best_move);
        Ok(best_score)
    }

    fn search_captures(&mut self, mut alpha: Score, beta: Score) -> Score {
        self.seldepth = self.seldepth.max(self.depth_from_root);
        self.total_nodes += 1;

        let alpha_orig = alpha;

        let mut best_score = evaluation(&self.board) * self.board.active_side;
        alpha = alpha.max(best_score);
        if alpha >= beta {
            return beta;
        }
        let mut tt_move = None;
        if self.depth_from_root > 0
            && let Some(entry) = self.transposition_table.get(self.board.zobrist)
        {
            tt_move = entry.mov.opt().filter(|mov| mov.flags().is_capture());
            if let Some(score) = entry.score(alpha, beta, 0) {
                return score;
            }
        }

        let mut encountered_legal_move = false;
        let mut moves = MoveList::default();
        let mut best_move = None;

        while let Some(mov) = moves.next::<true>(self, tt_move, None) {
            if !self.board.is_legal(mov) {
                continue;
            }
            encountered_legal_move = true;
            let unmake = self.board.make_move(mov);
            self.depth_from_root += 1;
            let score = -self.search_captures(-beta, -alpha);
            self.depth_from_root -= 1;
            self.board.unmake_move(unmake);

            if score > best_score {
                best_move = Some(mov);
                best_score = score;
            }

            alpha = alpha.max(score);

            if score >= beta {
                break;
            }
        }

        if !encountered_legal_move {
            let legal_moves =
                self.board.pseudolegal_moves().iter().any(|&mov| self.board.is_legal(mov));
            if !legal_moves {
                return if self.board.in_check() {
                    -Score::mate_in_ply(self.depth_from_root)
                } else {
                    Score(0)
                };
            }
        }

        let nodetype = if best_score <= alpha_orig {
            Nodetype::Alpha
        } else if best_score >= beta {
            Nodetype::Beta
        } else {
            Nodetype::Exact
        };
        self.transposition_table.insert(self.board.zobrist, 0, best_score, nodetype, best_move);

        best_score
    }

    fn should_null_move_heuristic(&self, depth: u8) -> bool {
        if self.depth_from_root < 3 || depth < 3 {
            return false;
        }
        // try avoid zugzwang issue
        if phase(&self.board).endgame().as_float() > 0.9 {
            return false;
        }
        !self.board.in_check()
    }
}
