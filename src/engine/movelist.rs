use arrayvec::ArrayVec;

use super::Engine;
use crate::core::{r#move::Move, moves::Moves};

#[derive(Default)]
pub struct MoveList {
    moves: Moves,
    bad_captures: ArrayVec<Move, 80>,
    state: State,
}

#[derive(Default)]
enum State {
    #[default]
    Hash,
    Capture,
    Killer,
    Quiet,
    BadCaptures,
    Finished,
}

impl MoveList {
    pub fn next<const CAPTURES_ONLY: bool>(
        &mut self,
        engine: &mut Engine,
        tt_move: Option<Move>,
        killer: Option<Move>,
    ) -> Option<Move> {
        loop {
            if let Some(mov) = self.moves.pop() {
                break Some(mov);
            }
            match self.state {
                State::Hash => {
                    self.state = State::Capture;
                    if let Some(mov) = tt_move
                        && engine.board.is_pseudolegal(mov)
                    {
                        break Some(mov);
                    }
                }
                State::Capture => {
                    self.state = if CAPTURES_ONLY { State::BadCaptures } else { State::Killer };
                    self.moves = engine.board.pseudolegal_capture_moves();
                    self.remove_tt_killer(tt_move, killer);
                    let [good, bad] = engine.order_moves_capture(&self.moves);
                    self.moves.clear();
                    self.moves.extend(good);
                    self.bad_captures = bad;
                }
                State::Killer => {
                    self.state = State::Quiet;
                    if let Some(mov) = killer
                        && engine.board.is_pseudolegal(mov)
                    {
                        break Some(mov);
                    }
                }
                State::Quiet => {
                    self.state = State::BadCaptures;
                    self.moves = engine.board.pseudolegal_quiet_moves();
                    self.remove_tt_killer(tt_move, killer);
                    engine.order_moves_history(&mut self.moves);
                }
                State::BadCaptures => {
                    self.state = State::Finished;
                    self.moves.extend(self.bad_captures.clone());
                }
                State::Finished => break None,
            }
        }
    }

    fn remove_tt_killer(&mut self, tt_move: Option<Move>, killer: Option<Move>) {
        let mut remove = |mov| {
            if let Some(mov) = mov
                && let Some(index) = self.moves.iter().position(|&m| m == mov)
            {
                self.moves.swap_remove(index);
            }
        };
        remove(tt_move);
        remove(killer);
    }
}
