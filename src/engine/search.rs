use std::time::Instant;

use movegen::FullGen;

use super::{transposition::Nodetype, Engine};
use crate::{
    engine::score::Eval,
    prelude::*,
    uci::{Info, Score, UciResponse},
};

impl Engine {
    pub fn search(&mut self) -> Move {
        self.time_started = Instant::now();
        self.total_nodes = 0;
        self.effective_nodes = 0;
        self.force_cancelled = false;
        self.transposition_table.num_hits = 0;

        let beta = Eval::INFINITY.0;

        let mut best_move = Move::NULL;
        for depth in 1..=255 {
            if self.time_started.elapsed() > self.time_available / 2 {
                break;
            }
            self.only_pv_nodes = true;
            let mut new_pv = Moves::new();
            let score = self.negamax(-beta, beta, depth, &mut new_pv, None).0;
            if self.is_cancelled() {
                break;
            }
            self.pv = new_pv.into_iter().rev().collect();
            best_move = *self.pv.first().unwrap_or(&best_move);
            self.effective_nodes = self.total_nodes;
            self.depth_reached = depth;

            let is_checkmate = score.abs() >= Eval::INFINITY.0;

            let score = if is_checkmate {
                Score::Mate { mate: depth as i32 / 2 * score.signum() }
            } else {
                Score::Centipawns { cp: score, bounds: None }
            };

            let time_taken = self.time_started.elapsed();
            let info = Info {
                depth: Some(depth as u32),
                score: Some(score),
                nodes: Some(self.total_nodes),
                time: Some(time_taken),
                nps: Some((self.total_nodes as f64 / time_taken.as_secs_f64()) as u32),
                pv: Some(self.pv.clone()),
                ..Info::default()
            };
            tracing::info!("{info}");
            println!("{}", UciResponse::Info(Box::new(info)));

            if is_checkmate {
                break;
            }
        }
        best_move
    }
    fn seen_position(&self) -> bool {
        self.seen_positions.iter().filter(|&&sq| sq == self.board.zobrist).count() > 1
    }
    #[allow(clippy::too_many_lines)]
    pub(crate) fn negamax(
        &mut self,
        mut alpha: i32,
        beta: i32,
        depth: u8,
        pline: &mut Moves,
        killer_move: Option<Move>,
    ) -> (i32, Option<Move>) {
        if self.seen_position() {
            return (0, None);
        }
        if self.depth_from_root > 0 {
            if let Some(eval) = self.transposition_table.get(&self.board, alpha, beta, depth) {
                return (eval, None);
            }
        }
        if depth == 0 {
            self.only_pv_nodes = false;
            return (self.negamax_search_all_captures(alpha, beta), None);
        }
        if self.depth_from_root > 0 {
            self.total_nodes += 1;
        }

        'null: {
            if self.depth_from_root < 3 || depth < 3 {
                break 'null;
            }
            // try avoid zugzwang issue
            if self.phase().endgame().0 > 0.9 {
                break 'null;
            }
            if self.board.in_check() {
                break 'null;
            }
            let unmake = self.board.make_null_move();
            self.depth_from_root += 1;
            let score = -self.negamax(-beta, -alpha, depth - 3, &mut Moves::new(), None).0;
            self.depth_from_root -= 1;
            self.board.unmake_null_move(unmake);
            if score >= beta {
                self.transposition_table.insert(
                    &self.board,
                    &self.seen_positions,
                    depth - 2,
                    beta,
                    Nodetype::Beta,
                    0,
                );
                return (beta, None);
            }
        }

        let mut moves = MoveGenerator::<FullGen>::new(&mut self.board).gen_pseudolegal_moves();
        let mut encountered_legal_move = false;

        self.order_moves(&mut moves, killer_move);
        let mut nodetype = Nodetype::Alpha;

        let curr_nodes = self.total_nodes;
        let mut killer_move = None;
        for mov in moves {
            if !MoveGenerator::<FullGen>::new(&mut self.board).is_legal(mov) {
                continue;
            }
            let mut line = Moves::new();
            encountered_legal_move = true;
            let unmake = self.board.make_move(mov);
            self.seen_positions.push(self.board.zobrist);
            self.depth_from_root += 1;

            let (score, chosen_move) = self.negamax(-beta, -alpha, depth - 1, &mut line, killer_move);
            let score = -score;
            killer_move = chosen_move;

            self.depth_from_root -= 1;
            self.seen_positions.pop();
            self.board.unmake_move(unmake);
            if self.is_cancelled() {
                return (0, None);
            }
            if score > alpha {
                line.push(mov);
                *pline = line;
                alpha = score;
                nodetype = Nodetype::Exact;
            }
            if score >= beta {
                self.transposition_table.insert(
                    &self.board,
                    &self.seen_positions,
                    depth,
                    beta,
                    Nodetype::Beta,
                    self.total_nodes - curr_nodes,
                );
                return (beta, Some(mov));
            }
        }

        if !encountered_legal_move {
            if self.board.in_check() {
                return (-Eval::MATE.0, None);
            }
            return (0, None);
        }

        self.transposition_table.insert(
            &self.board,
            &self.seen_positions,
            depth,
            alpha,
            nodetype,
            self.total_nodes - curr_nodes,
        );
        (alpha, None)
    }

    fn negamax_search_all_captures(&mut self, mut alpha: i32, beta: i32) -> i32 {
        self.total_nodes += 1;

        let eval = self.evaluate();
        if eval >= beta {
            return beta;
        }
        alpha = alpha.max(eval);

        let mut moves = self.board.gen_pseudolegal_capture_moves();
        self.order_moves(&mut moves, None);

        let mut encountered_legal_move = false;
        for mov in moves {
            if !MoveGenerator::<FullGen>::new(&mut self.board).is_legal(mov) {
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
            let mut movegen = MoveGenerator::<FullGen>::new(&mut self.board);
            let legal_moves = movegen.gen_pseudolegal_moves().iter().any(|&mov| movegen.is_legal(mov));
            if !legal_moves {
                return if self.board.in_check() { -Eval::MATE.0 } else { 0 };
            }
        }

        alpha
    }
}
