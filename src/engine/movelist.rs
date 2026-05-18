use super::Engine;
use crate::core::{r#move::Move, moves::Moves};

#[derive(Default)]
pub struct MoveList {
    moves: Moves,
    state: State,
}

#[derive(Default)]
enum State {
    #[default]
    Hash,
    Capture,
    Killer,
    Quiet,
    Finished,
}

impl MoveList {
    pub fn next<const CAPTURES_ONLY: bool>(
        &mut self,
        engine: &mut Engine,
        tt_move: Option<Move>,
        killer: Option<Move>,
        depth: u8,
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
                    self.state = if CAPTURES_ONLY { State::Finished } else { State::Killer };
                    self.moves = engine.board.pseudolegal_capture_moves();
                    self.remove_tt_killer(tt_move, killer);
                    engine.order_moves_capture(depth, &mut self.moves);
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
                    self.state = State::Finished;
                    self.moves = engine.board.pseudolegal_quiet_moves();
                    self.remove_tt_killer(tt_move, killer);
                    engine.order_moves_history(&mut self.moves);
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
