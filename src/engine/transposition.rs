use crate::prelude::*;

#[derive(Clone)]
pub struct TranspositionTable<T> {
    inner: Box<[Option<Entry<T>>]>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Nodetype {
    Exact,
    Alpha,
    Beta,
}

impl<T> TranspositionTable<T> {
    #[must_use]
    pub fn from_mb(mb: usize) -> Self {
        Self {
            inner: std::iter::repeat_with(|| None)
                .take(mb * 1024 * 1024 / size_of::<Entry<T>>())
                .collect(),
        }
    }

    #[must_use]
    pub fn get(&self, zobrist: Zobrist) -> Option<&Entry<T>> {
        self.inner[(zobrist.0 % self.inner.len() as u64) as usize]
            .as_ref()
            .and_then(|entry| (entry.zobrist == zobrist).then_some(entry))
    }

    #[must_use]
    fn get_mut(&mut self, zobrist: Zobrist) -> &mut Option<Entry<T>> {
        &mut self.inner[(zobrist.0 % self.inner.len() as u64) as usize]
    }

    pub fn insert(
        &mut self,
        zobrist: Zobrist,
        depth: u8,
        eval: Score,
        nodetype: Nodetype,
        extra: T,
    ) {
        let entry = Entry { zobrist, eval, nodetype, depth, extra };
        match self.get_mut(zobrist) {
            Some(occupied) => {
                if occupied.depth <= depth {
                    *occupied = entry;
                }
            }
            vacant @ None => *vacant = Some(entry),
        }
    }
}

#[derive(Clone)]
pub struct Entry<T> {
    pub zobrist: Zobrist,
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
