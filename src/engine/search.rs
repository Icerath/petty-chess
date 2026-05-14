use std::time::Instant;

use super::{Engine, evaluation::evaluate, phase::phase, transposition::Nodetype};
use crate::{
    engine::score::Score,
    prelude::*,
    uci::{Info, UciResponse},
};

impl Engine {
    pub fn search(&mut self) -> Move {
        let time_started = Instant::now();
        self.total_nodes = 0;

        self.killer.clear();
        self.killer.extend([None; 64]);

        let mut best_move = *self.board.legal_moves().first().unwrap_or(&Move::NULL);

        for depth in 1.. {
            let mut new_pv = Moves::new();
            let score = self.negamax(-Score::MAX, Score::MAX, depth, &mut new_pv);
            self.total_nodes -= 1;
            if self.is_cancelled() {
                break;
            }
            self.pv = new_pv.iter().copied().rev().collect();
            best_move = *self.pv.first().unwrap_or(&best_move);

            let mate = score.mate();

            let time_taken = time_started.elapsed();
            let info = Info {
                depth: Some(depth as u32),
                score: Some(score),
                nodes: Some(self.total_nodes),
                time: Some(time_taken),
                nps: Some((self.total_nodes as f64 / time_taken.as_secs_f64()) as u32),
                pv: Some(self.pv.clone()),
                ..Info::default()
            };
            #[cfg(feature = "tracing")]
            tracing::info!("{info}");
            println!("{}", UciResponse::Info(Box::new(info)));

            if mate.is_some() {
                break;
            }

            let skip_time = 2.8 - phase(&self.board).endgame().as_float();
            if time_started.elapsed().mul_f32(skip_time) > self.time_available {
                break;
            }
        }
        #[cfg(feature = "tracing")]
        tracing::info!("Tablesize: {}Mb", self.transposition_table.mb());
        best_move
    }

    fn seen_position(&self) -> bool {
        self.seen_positions.iter().filter(|&&sq| sq == self.board.zobrist).count() > 1
    }

    pub(crate) fn negamax(
        &mut self,
        mut alpha: Score,
        beta: Score,
        depth: u8,
        pline: &mut Moves,
    ) -> Score {
        if self.depth_from_root != 0 && self.seen_position() {
            return Score(0);
        }

        let mut tt_move = None;
        if self.depth_from_root > 0
            && let Some(entry) = self.transposition_table.get(&self.board)
        {
            tt_move = entry.extra;
            if let Some(score) = entry.score(alpha, beta, depth) {
                if let Some(tt_move) = tt_move {
                    pline.push(tt_move);
                }
                return score;
            }
        }
        if depth == 0 {
            return self.negamax_search_all_captures(alpha, beta);
        }
        self.total_nodes += 1;

        if self.should_null_move_heuristic(depth) {
            let unmake = self.board.make_null_move();
            self.depth_from_root += 1;
            let score = -self.negamax(-beta, -alpha, depth - 3, &mut Moves::new());
            self.depth_from_root -= 1;
            self.board.unmake_null_move(unmake);
            if score >= beta {
                return beta;
            }
        }

        let mut moves = self.board.pseudolegal_moves();
        let mut encountered_legal_move = false;

        self.order_moves(&mut moves, self.killer[self.depth_from_root as usize], tt_move);
        let mut nodetype = Nodetype::Alpha;

        let mut best_move = None;
        for mov in moves {
            if self.is_cancelled() {
                return Score(0);
            }
            if !self.board.is_legal(mov) {
                continue;
            }
            let mut line = Moves::new();
            encountered_legal_move = true;
            let unmake = self.board.make_move(mov);
            self.seen_positions.push(self.board.zobrist);

            let extension = self.board.in_check() as u8;

            self.depth_from_root += 1;
            let score = -self.negamax(-beta, -alpha, depth - 1 + extension, &mut line);
            self.depth_from_root -= 1;

            self.seen_positions.pop();
            self.board.unmake_move(unmake);

            if score > alpha {
                line.push(mov);
                *pline = line;
                alpha = score;
                nodetype = Nodetype::Exact;
                best_move = Some(mov);
            }
            if score >= beta {
                self.killer[self.depth_from_root as usize] = Some(mov);
                self.transposition_table.insert(
                    &self.board,
                    &self.seen_positions,
                    depth,
                    beta,
                    Nodetype::Beta,
                    Some(mov),
                );
                return beta;
            }
        }

        if !encountered_legal_move {
            if self.board.in_check() {
                return -Score::mate_in_moves(self.depth_from_root.cast_signed() / 2 + 1);
            }
            return Score(0);
        }

        self.transposition_table.insert(
            &self.board,
            &self.seen_positions,
            depth,
            alpha,
            nodetype,
            best_move,
        );
        alpha
    }

    fn negamax_search_all_captures(&mut self, mut alpha: Score, beta: Score) -> Score {
        self.total_nodes += 1;

        let eval = evaluate(&self.board);
        if eval >= beta {
            return beta;
        }
        alpha = alpha.max(eval);

        let mut moves = self.board.pseudolegal_capture_moves();
        self.order_moves(&mut moves, None, None);

        let mut encountered_legal_move = false;
        for mov in moves {
            if !self.board.is_legal(mov) {
                continue;
            }
            if self.is_cancelled() {
                return Score(0);
            }
            encountered_legal_move = true;
            let unmake = self.board.make_move(mov);
            self.depth_from_root += 1;
            let score = -self.negamax_search_all_captures(-beta, -alpha);
            self.depth_from_root -= 1;
            self.board.unmake_move(unmake);

            if score >= beta {
                return beta;
            }
            alpha = alpha.max(score);
        }

        if !encountered_legal_move {
            let legal_moves =
                self.board.pseudolegal_moves().iter().any(|&mov| self.board.is_legal(mov));
            if !legal_moves {
                return if self.board.in_check() {
                    -Score::mate_in_moves(self.depth_from_root.cast_signed() / 2 + 1)
                } else {
                    Score(0)
                };
            }
        }

        alpha
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
