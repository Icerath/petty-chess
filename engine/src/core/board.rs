use core::fmt;
use std::ops::{Index, IndexMut};

use crate::prelude::*;

pub struct Board {
    pub pieces: [Option<Piece>; 64],
    pub active_colour: Colour,
    pub can_castle: CanCastle,
    pub en_passant_target_square: Option<Pos>,
    pub halfmove_clock: u8,
    pub fullmove_counter: u16,
}

impl Index<Pos> for Board {
    type Output = Option<Piece>;
    fn index(&self, index: Pos) -> &Self::Output {
        &self.pieces[index.0 as usize]
    }
}

impl IndexMut<Pos> for Board {
    fn index_mut(&mut self, index: Pos) -> &mut Self::Output {
        &mut self.pieces[index.0 as usize]
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_fen())
    }
}
