use std::time::Duration;

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum UciMessage {
    Uci,
    Debug(bool),
    Isready,
    Setoption { id: String, value: Option<String> },
    Register(Registration),
    Ucinewgame,
    Position { fen: String, moves: Vec<Move> },
    Go(GoCommand),
    Stop,
    PonderHit,
    Quit,
    Perft { depth: Option<u32> },
    Display,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum Registration {
    Later,
    Now { name: Option<String>, code: Option<String> },
}

#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct GoCommand {
    pub searchmoves: Option<Vec<Move>>,
    pub time_control: TimeControl,
    pub depth: Option<u32>,
    pub nodes: Option<u64>,
    pub mate: Option<u32>,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub enum TimeControl {
    Ponder,
    TimeLeft {
        base: [Duration; Side::LEN],
        incr: [Duration; Side::LEN],
        moves_to_go: Option<u32>,
    },
    MoveTime(Duration),
    #[default]
    Infinite,
}
