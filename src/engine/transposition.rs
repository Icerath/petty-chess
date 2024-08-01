use std::collections::HashMap;

use crate::prelude::*;

#[derive(Default)]
pub struct TranspositionTable {
    inner: HashMap<Zobrist, Entry>,
    pub num_hits: u64,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Nodetype {
    Exact,
    Alpha,
    Beta,
}

impl TranspositionTable {
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }
    pub fn clear(&mut self) {
        self.inner.clear();
    }
    #[must_use]
    #[inline]
    pub fn get(&mut self, board: &Board, alpha: i32, beta: i32, depth: u8) -> Option<i32> {
        self.get_entry(board, alpha, beta, depth).map(|entry| entry.eval)
    }
    #[must_use]
    #[inline]
    pub fn get_entry(&mut self, board: &Board, alpha: i32, beta: i32, depth: u8) -> Option<&Entry> {
        let entry = self.inner.get(&board.zobrist)?;
        if entry.depth < depth {
            return None;
        }
        if entry.board != CoreBoard::from(board) {
            return None;
        }
        if (entry.nodetype == Nodetype::Exact)
            || (entry.nodetype == Nodetype::Alpha && entry.eval <= alpha)
            || (entry.nodetype == Nodetype::Beta && entry.eval >= beta)
        {
            self.num_hits += entry.treesize;
            return Some(entry);
        }
        None
    }
    #[inline]
    pub fn insert(&mut self, board: &Board, depth: u8, eval: i32, nodetype: Nodetype, treesize: u64) {
        let entry = Entry { board: CoreBoard::from(board), eval, nodetype, depth, treesize };
        self.inner.insert(board.zobrist, entry);
    }
}

pub struct Entry {
    board: CoreBoard,
    pub eval: i32,
    pub nodetype: Nodetype,
    pub depth: u8,
    pub treesize: u64,
}

#[derive(PartialEq)]
struct CoreBoard {
    pieces: [Option<Piece>; 64],
    en_passant: Option<Pos>,
    can_castle: CanCastle,
    active_colour: Colour,
}

impl From<&Board> for CoreBoard {
    fn from(board: &Board) -> Self {
        Self {
            pieces: board.pieces,
            en_passant: board.en_passant_target_square,
            can_castle: board.can_castle,
            active_colour: board.active_colour,
        }
    }
}
