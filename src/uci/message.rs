use std::time::Duration;

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum UciMessage {
    Uci,
    Debug(bool),
    Isready,
    Setoption { id: String, value: Option<String> },
    Register(Registration),
    Ucinewgame,
    Position { fen: String, moves: Moves },
    Go(GoCommand),
    Stop,
    PonderHit,
    Quit,
    Perft { depth: Option<u32> },
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Registration {
    Later,
    Now { name: Option<String>, code: Option<String> },
}

#[derive(Default, Debug, Clone, PartialEq, PartialOrd)]
pub struct GoCommand {
    pub searchmoves: Option<Moves>,
    pub time_control: TimeControl,
    pub depth: Option<u32>,
    pub nodes: Option<u64>,
    pub mate: Option<u32>,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum TimeControl {
    Ponder,
    TimeLeft {
        wtime: Duration,
        btime: Duration,
        wincr: Duration,
        bincr: Duration,
        moves_to_go: Option<u32>,
    },
    MoveTime(Duration),
    #[default]
    Infinite,
}
