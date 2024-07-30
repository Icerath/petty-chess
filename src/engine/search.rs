use std::time::Instant;

use rand::prelude::*;

use super::Engine;
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

        let beta = i32::MAX;

        let mut moves = self.board.gen_legal_moves();
        let mut final_best_moves = Moves::new();

        'outer: for depth in 1..=255 {
            self.order_moves(&mut moves, &final_best_moves);
            let mut best_moves = Moves::new();
            let mut alpha = -beta;

            for &mov in &moves {
                let unmake = self.board.make_move(mov);
                let score = -self.negamax(-beta, beta, depth - 1);
                self.board.unmake_move(unmake);

                if self.is_cancelled() {
                    if depth == 1 {
                        dbg!("Cancelled during depth 1 search");
                    }
                    break 'outer;
                }

                if score > alpha {
                    best_moves.clear();
                }
                if score >= alpha {
                    best_moves.push(mov);
                    alpha = score;
                }
            }
            self.effective_nodes = self.total_nodes;
            self.depth_reached = depth;
            final_best_moves = best_moves;

            let absolute_eval = alpha * self.board.active_colour.positive();
            let info = Info {
                depth: Some(depth as u32),
                score: Some(Score::Centipawns { cp: absolute_eval, bounds: None }),
                nodes: Some(self.total_nodes),
                time: Some(self.time_started.elapsed()),
                currmove: Some(final_best_moves[0]),
                ..Info::default()
            };
            tracing::info!("{info}");
            println!("{}", UciResponse::Info(Box::new(info)));
        }
        final_best_moves.choose(&mut rand::thread_rng()).copied().unwrap_or(moves[0])
    }

    pub(crate) fn negamax(&mut self, mut alpha: i32, beta: i32, depth: u8) -> i32 {
        if self.board.seen_position() > 1 {
            return 0;
        }
        if depth == 0 {
            return self.negamax_search_all_captures(alpha, beta);
        }

        let mut moves = self.board.gen_legal_moves();
        self.order_moves(&mut moves, &[]);

        for mov in moves {
            let unmake = self.board.make_move(mov);
            let score = -self.negamax(-beta, -alpha, depth - 1);
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

    fn negamax_search_all_captures(&mut self, mut alpha: i32, beta: i32) -> i32 {
        let mut moves = self.board.gen_capture_moves();
        self.order_moves(&mut moves, &[]);

        let eval = self.evaluate();
        if eval >= beta {
            return beta;
        }
        alpha = alpha.max(eval);

        for mov in moves {
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
