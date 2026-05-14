use std::{
    collections::{HashMap, hash_map::Entry as HashEntry},
    hash::{BuildHasherDefault, Hasher},
};

use crate::prelude::*;

#[derive(Default, Clone)]
pub struct TranspositionTable<T> {
    inner: HashMap<Zobrist, Entry<T>, BuildHasherDefault<NoHasher>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Nodetype {
    Exact,
    Alpha,
    Beta,
}

impl<T> TranspositionTable<T> {
    #[must_use]
    pub fn mb(&self) -> usize {
        (self.inner.capacity() * (size_of::<Zobrist>() + size_of::<Entry<T>>())) / 1_000_000
    }

    #[must_use]
    pub fn get(&mut self, board: &Board) -> Option<&Entry<T>> {
        self.inner.get(&board.zobrist)
    }

    pub fn insert(&mut self, board: &Board, depth: u8, eval: Score, nodetype: Nodetype, extra: T) {
        let entry = Entry { eval, nodetype, depth, extra };
        match self.inner.entry(board.zobrist) {
            HashEntry::Occupied(mut occupied) => {
                if occupied.get().depth <= depth {
                    occupied.insert(entry);
                }
            }
            HashEntry::Vacant(vacant) => _ = vacant.insert(entry),
        }
    }
}

#[derive(Clone)]
pub struct Entry<T> {
    pub eval: Score,
    pub nodetype: Nodetype,
    pub depth: u8,
    pub extra: T,
}

impl<T> Entry<T> {
    pub fn score(&self, alpha: Score, beta: Score, depth: u8) -> Option<Score> {
        if self.depth < depth {
            return None;
        }
        if (self.nodetype == Nodetype::Exact)
            || (self.nodetype == Nodetype::Alpha && self.eval <= alpha)
            || (self.nodetype == Nodetype::Beta && self.eval >= beta)
        {
            return Some(self.eval);
        }
        None
    }
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
