use std::{
    collections::HashMap,
    hash::{BuildHasherDefault, Hasher},
};

use crate::prelude::*;

#[derive(Default, Clone)]
pub struct TranspositionTable {
    inner: HashMap<Zobrist, Entry, BuildHasherDefault<NoHasher>>,
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
    pub fn get(&mut self, board: &Board, alpha: i32, beta: i32, depth: u8) -> Option<i32> {
        self.get_entry(board, alpha, beta, depth).map(|entry| entry.eval)
    }

    #[must_use]
    pub fn get_entry(&mut self, board: &Board, alpha: i32, beta: i32, depth: u8) -> Option<&Entry> {
        let entry = self.inner.get(&board.zobrist)?;
        if entry.depth < depth {
            return None;
        }
        if (entry.nodetype == Nodetype::Exact)
            || (entry.nodetype == Nodetype::Alpha && entry.eval <= alpha)
            || (entry.nodetype == Nodetype::Beta && entry.eval >= beta)
        {
            self.num_hits += 1;
            return Some(entry);
        }
        None
    }

    pub fn insert(
        &mut self,
        board: &Board,
        seen_positions: &[Zobrist],
        depth: u8,
        eval: i32,
        nodetype: Nodetype,
        treesize: u64,
    ) {
        if eval.abs() == Eval::MATE.0 {
            return;
        }
        if seen_positions.iter().filter(|&&sq| sq == board.zobrist).count() > 1 {
            return;
        }
        let entry = Entry { eval, nodetype, depth, treesize };
        self.inner.insert(board.zobrist, entry);
    }
}

#[derive(Clone)]
pub struct Entry {
    pub eval: i32,
    pub nodetype: Nodetype,
    pub depth: u8,
    pub treesize: u64,
}

#[derive(Default)]
struct NoHasher(u64);

impl Hasher for NoHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, _bytes: &[u8]) {
        unreachable!();
    }

    fn write_u64(&mut self, i: u64) {
        self.0 = i;
    }
}
