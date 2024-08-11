use std::time::Instant;

use movegen::{CapturesOnly, FullGen};

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

        let beta = Self::mate_score();
        let mut moves = self.board.gen_legal_moves();
        if moves.is_empty() {
            return Move::NULL;
        }

        'outer: for depth in 1..=255 {
            if self.time_started.elapsed() > self.time_available / 2 {
                break;
            }
            self.only_pv_nodes = true;
            self.order_moves(&mut moves, None);
            if let Some(&mov) = self.pv.get(self.depth_from_root as usize) {
                if let Some(mov_index) = moves.iter().position(|&m| m == mov) {
                    moves.remove(mov_index);
                    moves.insert(0, mov);
                }
            }

            let mut new_pv = Moves::new();

            let mut alpha = -beta;
            let mut node_type = Nodetype::Alpha;

            let curr_nodes = self.total_nodes;
            for &mov in &moves {
                let mut line = Moves::new();
                let unmake = self.board.make_move(mov);
                self.seen_positions.push(self.board.zobrist);
                self.depth_from_root += 1;
                let score = -self.negamax(-beta, -alpha, depth - 1, &mut line, None).0;
                self.only_pv_nodes = false;
                self.depth_from_root -= 1;
                self.seen_positions.pop();
                self.board.unmake_move(unmake);

                if self.is_cancelled() {
                    break 'outer;
                }

                if score > alpha {
                    line.push(mov);
                    new_pv = line;
                    alpha = score;
                    node_type = Nodetype::Exact;
                }
            }
            new_pv.reverse();
            self.pv = new_pv;
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
                pv: Some(self.pv.clone()),
                ..Info::default()
            };
            tracing::info!("{info}");
            println!("{}", UciResponse::Info(Box::new(info)));

            if is_checkmate {
                break;
            }
        }
        self.pv[0]
    }
    fn seen_position(&self) -> bool {
        self.seen_positions.iter().filter(|&&pos| pos == self.board.zobrist).count() > 1
    }
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
        if let Some(eval) = self.transposition_table.get(&self.board, alpha, beta, depth) {
            return (eval, None);
        }
        if depth == 0 {
            self.only_pv_nodes = false;
            return (self.negamax_search_all_captures(alpha, beta), None);
        }
        self.total_nodes += 1;

        let mut moves = MoveGenerator::<FullGen>::new(&mut self.board).gen_pseudolegal_moves();
        let mut encountered_legal_move = false;

        self.order_moves(&mut moves, killer_move);
        let mut nodetype = Nodetype::Alpha;

        let curr_nodes = self.total_nodes;
        let mut killer_move = None;
        for (i, &mov) in moves.iter().enumerate() {
            _ = i;
            if !MoveGenerator::<FullGen>::new(&mut self.board).is_legal(mov) {
                continue;
            }
            let mut line = Moves::new();
            encountered_legal_move = true;
            let unmake = self.board.make_move(mov);
            self.seen_positions.push(self.board.zobrist);
            self.depth_from_root += 1;

            // if depth > 1 && i > moves.len() / 2 {
            //     let score = -self.negamax(-beta, -alpha, depth - 2, &mut line, killer_move).0;
            //     self.only_pv_nodes = false;
            //     if score <= alpha {
            //         self.depth_from_root -= 1;
            //         self.seen_positions.pop();
            //         self.board.unmake_move(unmake);
            //         continue;
            //     }
            // }

            let (score, chosen_move) = self.negamax(-beta, -alpha, depth - 1, &mut line, killer_move);
            let score = -score;
            killer_move = chosen_move;

            self.depth_from_root -= 1;
            self.seen_positions.pop();
            self.board.unmake_move(unmake);
            if self.is_cancelled() {
                return (alpha, None);
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
            if score > alpha {
                line.push(mov);
                *pline = line;
                alpha = score;
                nodetype = Nodetype::Exact;
            }
        }

        if !encountered_legal_move {
            if MoveGenerator::<CapturesOnly>::new(&mut self.board)
                .attack_map()
                .contains(self.board.active_king_pos)
            {
                return (-Self::mate_score(), None);
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
                return alpha;
            }

            if score >= beta {
                return beta;
            }
            alpha = alpha.max(score);
        }

        if !encountered_legal_move {
            let mut movegen = MoveGenerator::<FullGen>::new(&mut self.board);
            let legal_moves = movegen
                .gen_pseudolegal_moves()
                .iter()
                .filter(|mov| !mov.flags().is_capture())
                .any(|&mov| movegen.is_legal(mov));

            let is_check = movegen.attack_map().contains(self.board.active_king_pos);
            if !legal_moves && is_check {
                // TODO - doesn't product a correct `mate in` score
                return -Self::mate_score();
            }
        }

        alpha
    }
}
