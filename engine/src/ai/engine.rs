use crate::prelude::*;

#[derive(Default)]
pub struct Engine {
    pub board: Board,
}

impl Engine {
    pub fn search(&mut self) -> Move {
        self.board.gen_legal_moves()[0]
    }
}
