use std::time::Instant;

use super::{transposition::Nodetype, Engine};
use crate::{
    prelude::*,
    uci::{Info, Score, UciResponse},
};

impl Engine {
    #[allow(clippy::unnecessary_wraps)]
    pub fn search(&mut self) -> Move {
        self.time_started = Instant::now();
        self.total_nodes = 0;
        self.effective_nodes = 0;
        self.force_cancelled = false;
        self.transposition_table.num_hits = 0;
        self.seen_positions = vec![self.board.zobrist];

        let beta = Self::mate_score();
        let mut moves = self.board.gen_legal_moves();
        let mut final_best_moves = Moves::new();

        'outer: for depth in 1..=255 {
            self.order_moves(&mut moves, &final_best_moves);
            let mut best_moves = Moves::new();
            let mut alpha = -beta;
            let mut node_type = Nodetype::Alpha;

            let curr_nodes = self.total_nodes;
            for &mov in &moves {
                let unmake = self.board.make_move(mov);
                self.seen_positions.push(self.board.zobrist);
                let score = -self.negamax(-beta, beta, depth - 1);
                self.seen_positions.pop();
                self.board.unmake_move(unmake);
                if self.is_cancelled() {
                    break 'outer;
                }

                if score > alpha {
                    best_moves.clear();
                }
                if score >= alpha {
                    best_moves.push(mov);
                    alpha = score;
                    node_type = Nodetype::Exact;
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
                }
            }
            self.transposition_table.insert(
                &self.board,
                &self.seen_positions,
                depth,
                alpha,
                node_type,
                self.total_nodes - curr_nodes,
            );
            self.effective_nodes = self.total_nodes;
            self.depth_reached = depth;
            final_best_moves = best_moves;

            let is_checkmate = alpha.abs() == Self::mate_score();

            let absolute_eval = alpha * self.board.active_colour.positive();
            let score = if is_checkmate {
                Score::Mate { mate: depth as i32 / 2 * alpha.signum() }
            } else {
                Score::Centipawns { cp: absolute_eval, bounds: None }
            };

            let info = Info {
                depth: Some(depth as u32),
                score: Some(score),
                nodes: Some(self.total_nodes),
                time: Some(self.time_started.elapsed()),
                currmove: Some(final_best_moves[0]),
                ..Info::default()
            };
            tracing::info!("{info}");
            println!("{}", UciResponse::Info(Box::new(info)));

            if is_checkmate {
                break;
            }
        }
        final_best_moves[0]
    }
    fn seen_position(&self) -> bool {
        self.seen_positions.iter().filter(|&&pos| pos == self.board.zobrist).count() > 1
    }
    pub(crate) fn negamax(&mut self, mut alpha: i32, beta: i32, depth: u8) -> i32 {
        if self.seen_position() {
            return 0;
        }
        if let Some(eval) = self.transposition_table.get(&self.board, alpha, beta, depth) {
            return eval;
        }
        if depth == 0 {
            return self.negamax_search_all_captures(alpha, beta);
        }

        let mut movegen = MoveGenerator::new(&mut self.board);
        let mut moves = movegen.gen_pseudolegal_moves();
        let attack_map = movegen.attack_map();
        let mut encountered_legal_move = false;

        self.order_moves(&mut moves, &[]);
        let mut nodetype = Nodetype::Alpha;

        let curr_nodes = self.total_nodes;
        for mov in moves {
            if !MoveGenerator::new(&mut self.board).is_legal(mov) {
                continue;
            }
            encountered_legal_move = true;
            let unmake = self.board.make_move(mov);
            self.seen_positions.push(self.board.zobrist);
            let score = -self.negamax(-beta, -alpha, depth - 1);
            self.seen_positions.pop();
            self.board.unmake_move(unmake);
            if self.is_cancelled() {
                return alpha;
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
                return beta;
            }
            if score > alpha {
                alpha = score;
                nodetype = Nodetype::Exact;
            }
            alpha = alpha.max(score);
        }

        if !encountered_legal_move {
            if attack_map.contains(self.board.active_king_pos) {
                return -Self::mate_score();
            }
            return 0;
        }

        self.transposition_table.insert(
            &self.board,
            &self.seen_positions,
            depth,
            alpha,
            nodetype,
            self.total_nodes - curr_nodes,
        );
        alpha
    }

    fn negamax_search_all_captures(&mut self, mut alpha: i32, beta: i32) -> i32 {
        if self.seen_position() {
            return 0;
        }
        if let Some(eval) = self.transposition_table.get(&self.board, alpha, beta, 0) {
            return eval;
        }
        let mut moves = self.board.gen_pseudolegal_capture_moves();
        self.order_moves(&mut moves, &[]);

        let eval = self.evaluate();
        if eval >= beta {
            return beta;
        }
        alpha = alpha.max(eval);

        for mov in moves {
            if !MoveGenerator::new(&mut self.board).is_legal(mov) {
                continue;
            }
            let unmake = self.board.make_move(mov);
            let score = -self.negamax_search_all_captures(-beta, -alpha);
            self.board.unmake_move(unmake);

            if self.is_cancelled() {
                return alpha;
            }

            if score >= beta {
                return beta;
            }
            alpha = alpha.max(score);
        }
        alpha
    }
}
