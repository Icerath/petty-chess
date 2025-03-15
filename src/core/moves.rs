use std::{
    fmt,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};

use super::r#move::Move;

// An array of moves with the unsafe invariant that there are no more than 256 moves
#[derive(Clone)]
pub struct Moves {
    array: [MaybeUninit<Move>; 256],
    len: u8,
}

impl Moves {
    #[must_use]
    pub const fn new() -> Self {
        Self { array: [MaybeUninit::uninit(); 256], len: 0 }
    }

    /// # Safety
    /// self.len must be < 255
    pub unsafe fn push_unchecked(&mut self, mov: Move) {
        debug_assert!(self.len < u8::MAX);
        self.array[self.len as usize].write(mov);
        self.len += 1;
    }

    /// Returns false if if failed to push the move
    pub fn push(&mut self, mov: Move) -> bool {
        if self.len == u8::MAX {
            return false;
        }
        unsafe { self.push_unchecked(mov) };
        true
    }

    #[must_use]
    pub const fn as_slice(&self) -> &[Move] {
        unsafe { std::slice::from_raw_parts(self.array.as_ptr().cast(), self.len as usize) }
    }

    #[must_use]
    pub const fn as_slice_mut(&mut self) -> &mut [Move] {
        unsafe { std::slice::from_raw_parts_mut(self.array.as_mut_ptr().cast(), self.len as usize) }
    }

    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Move) -> bool,
    {
        let mut new = Self::new();
        for mov in self.as_slice_mut() {
            if f(mov) {
                unsafe { new.push_unchecked(*mov) };
            }
        }
        *self = new;
    }
}

impl Default for Moves {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Moves {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self).finish()
    }
}

impl<'a> IntoIterator for &'a Moves {
    type IntoIter = Iter<'a>;
    type Item = &'a Move;

    fn into_iter(self) -> Self::IntoIter {
        Iter { moves: self, current: 0 }
    }
}

impl Deref for Moves {
    type Target = [Move];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl DerefMut for Moves {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_slice_mut()
    }
}

impl IntoIterator for Moves {
    type IntoIter = IntoIter;
    type Item = Move;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { moves: self, current: 0 }
    }
}

pub struct Iter<'a> {
    moves: &'a Moves,
    current: u8,
}

pub struct IntoIter {
    moves: Moves,
    current: u8,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.moves.len {
            return None;
        }
        let out = unsafe { self.moves.array[self.current as usize].assume_init_ref() };
        self.current += 1;
        Some(out)
    }
}

impl Iterator for IntoIter {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.moves.len {
            return None;
        }
        let out = unsafe { self.moves.array[self.current as usize].assume_init() };
        self.current += 1;
        Some(out)
    }
}
