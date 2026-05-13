use std::time::Instant;

use super::{Engine, evaluation::evaluate, phase::phase, transposition::Nodetype};
use crate::{
    engine::score::Eval,
    prelude::*,
    uci::{Info, Score, UciResponse},
};

impl Engine {
    pub fn search(&mut self) -> Move {
        let time_started = Instant::now();
        self.total_nodes = 0;

        self.killer.clear();
        self.killer.extend([None; 64]);

        let mut best_move = *self.board.legal_moves().first().unwrap_or(&Move::NULL);

        for depth in 1.. {
            self.only_pv_nodes = true;
            let mut new_pv = Moves::new();
            let score = self.negamax(-Eval::INFINITY.0, Eval::INFINITY.0, depth, &mut new_pv);
            self.total_nodes -= 1;
            if self.is_cancelled() {
                break;
            }
            self.pv = new_pv.iter().copied().rev().collect();
            best_move = *self.pv.first().unwrap_or(&best_move);

            let is_checkmate = score.abs() >= Eval::INFINITY.0;

            let score = if is_checkmate {
                Score::Mate { mate: depth as i32 / 2 * score.signum() }
            } else {
                Score::Centipawns { cp: score, bounds: None }
            };

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

            if is_checkmate {
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
        mut alpha: i32,
        beta: i32,
        depth: u8,
        pline: &mut Moves,
    ) -> i32 {
        if self.depth_from_root != 0 && self.seen_position() {
            return 0;
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
            self.only_pv_nodes = false;
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
                self.transposition_table.insert(
                    &self.board,
                    &self.seen_positions,
                    depth - 2,
                    beta,
                    Nodetype::Beta,
                    None,
                );
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
                return 0;
            }
            if !self.board.is_legal(mov) {
                continue;
            }
            let mut line = Moves::new();
            encountered_legal_move = true;
            let unmake = self.board.make_move(mov);
            self.seen_positions.push(self.board.zobrist);
            self.depth_from_root += 1;

            let extension = self.board.in_check() as u8;

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
                return -Eval::MATE.0;
            }
            return 0;
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

    fn negamax_search_all_captures(&mut self, mut alpha: i32, beta: i32) -> i32 {
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
            encountered_legal_move = true;
            let unmake = self.board.make_move(mov);
            self.depth_from_root += 1;
            let score = -self.negamax_search_all_captures(-beta, -alpha);
            self.depth_from_root -= 1;
            self.board.unmake_move(unmake);

            if self.is_cancelled() {
                return 0;
            }

            if score >= beta {
                return beta;
            }
            alpha = alpha.max(score);
        }

        if !encountered_legal_move {
            let legal_moves =
                self.board.pseudolegal_moves().iter().any(|&mov| self.board.is_legal(mov));
            if !legal_moves {
                return if self.board.in_check() { -Eval::MATE.0 } else { 0 };
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
