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
    Killer,
    Moves,
    Finished,
}

impl MoveList {
    pub fn next<const CAPTURES_ONLY: bool>(
        &mut self,
        board: &mut Board,
        tt_move: Option<Move>,
        killer: Option<Move>,
    ) -> Option<Move> {
        loop {
            if let Some(mov) = self.moves.pop() {
                break Some(mov);
            }
            match self.state {
                State::Hash => {
                    self.state = State::Killer;
                    if let Some(mov) = tt_move
                        && board.is_pseudolegal(mov)
                    {
                        break Some(mov);
                    }
                }
                State::Killer => {
                    self.state = State::Moves;
                    if let Some(mov) = killer
                        && board.is_pseudolegal(mov)
                    {
                        break Some(mov);
                    }
                }
                State::Moves => {
                    self.state = State::Finished;
                    self.moves = if CAPTURES_ONLY {
                        board.pseudolegal_capture_moves()
                    } else {
                        board.pseudolegal_moves()
                    };
                    self.remove_tt_killer(tt_move, killer);
                    order_moves(board, &mut self.moves);
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
    while let Some(next) = movelist.next::<false>(&mut board, None, None) {
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
    while let Some(next) = movelist.next::<true>(&mut board, None, None) {
        mlmoves.push(next);
    }
    order_moves(&mut board, &mut moves);
    assert_eq!(&*moves, mlmoves);
}
