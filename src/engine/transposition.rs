use crate::prelude::*;

#[derive(Clone)]
pub struct TranspositionTable {
    inner: Box<[Option<Entry>]>,
    len_minus_1: u64,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Nodetype {
    Exact,
    Alpha,
    Beta,
}

impl TranspositionTable {
    #[must_use]
    pub fn from_mb(mb: usize) -> Self {
        let inner: Box<[Option<Entry>]> = std::iter::repeat_with(|| None)
            .take((mb * 1024 * 1024 / size_of::<Entry>()).max(1).next_power_of_two())
            .collect();
        Self { len_minus_1: inner.len() as u64 - 1, inner }
    }

    fn index(&self, zobrist: Zobrist) -> usize {
        let index = (zobrist.0 & self.len_minus_1) as usize;
        unsafe { std::hint::assert_unchecked(index < self.inner.len()) };
        index
    }

    #[must_use]
    pub fn get(&self, zobrist: Zobrist) -> Option<&Entry> {
        let index = self.index(zobrist);
        self.inner[index].as_ref().and_then(|entry| (entry.zobrist == zobrist).then_some(entry))
    }

    pub fn insert(
        &mut self,
        zobrist: Zobrist,
        depth: u8,
        eval: Score,
        nodetype: Nodetype,
        mov: Option<Move>,
    ) {
        fn replace(old: &Entry, new: &Entry) -> bool {
            if old.depth <= new.depth {
                return true;
            }
            if old.depth == new.depth + 1 && (new.mov != Move::NULL && old.mov == Move::NULL) {
                return true;
            }
            false
        }
        let entry = Entry { zobrist, eval, nodetype, depth, mov: Move::from_opt(mov) };
        match &mut self.inner[self.index(zobrist)] {
            Some(occupied) if replace(occupied, &entry) => *occupied = entry,
            Some(_) => {}
            vacant @ None => *vacant = Some(entry),
        }
    }
}

const _: () = assert!(size_of::<Option<Entry>>() == 16);

#[repr(align(16))]
#[derive(Clone)]
pub struct Entry {
    pub zobrist: Zobrist,
    pub eval: Score,
    pub nodetype: Nodetype,
    pub depth: u8,
    pub mov: Move,
}

impl Entry {
    #[must_use]
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
