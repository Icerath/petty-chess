use super::move_ordering::order_moves;
use crate::core::{board::Board, r#move::Move, moves::Moves};

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
        board: &mut Board,
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
                        && board.is_pseudolegal(mov)
                    {
                        break Some(mov);
                    }
                }
                State::Capture => {
                    self.state = if CAPTURES_ONLY { State::Finished } else { State::Killer };
                    self.moves = board.pseudolegal_capture_moves();
                    self.remove_tt_killer(tt_move, killer);
                    order_moves(board, depth, &mut self.moves);
                }
                State::Killer => {
                    self.state = State::Quiet;
                    if let Some(mov) = killer
                        && board.is_pseudolegal(mov)
                    {
                        break Some(mov);
                    }
                }
                State::Quiet => {
                    self.state = State::Finished;
                    self.moves = board.pseudolegal_quiet_moves();
                    self.remove_tt_killer(tt_move, killer);
                    order_moves(board, depth, &mut self.moves);
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

#[test]
fn test_movelist() {
    let mut movelist = MoveList::default();
    let mut board = Board::start_pos();
    let mut moves = board.pseudolegal_moves();
    let mut mlmoves = vec![];
    while let Some(next) = movelist.next::<false>(&mut board, None, None, 0) {
        mlmoves.push(next);
    }
    moves.sort();
    mlmoves.sort();
    assert_eq!(&*moves, mlmoves);
}

#[test]
fn test_movelist_capture() {
    let mut movelist = MoveList::default();
    let mut board = Board::start_pos();
    let mut moves = board.pseudolegal_capture_moves();
    let mut mlmoves = vec![];
    while let Some(next) = movelist.next::<true>(&mut board, None, None, 0) {
        mlmoves.push(next);
    }
    order_moves(&mut board, 0, &mut moves);
    assert_eq!(&*moves, mlmoves);
}
