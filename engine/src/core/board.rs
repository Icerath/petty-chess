use crate::prelude::*;

pub struct Board {
    pub pieces: [Option<Piece>; 64],
    pub to_move: Colour,
}
