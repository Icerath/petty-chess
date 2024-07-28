use std::{fmt, time::Duration};

use crate::prelude::*;

pub enum UciResponse {
    Id { name: String, author: String },
    Uciok,
    Readyok,
    Bestmove { mov: Move, ponder: Option<Move> },
    Copyprotection(Requirement),
    Registration(Requirement),
    Info(Vec<Info>),
    Option { name: String, option: OptionType },
}

pub enum OptionType {
    Check { default: Option<bool> },
    Spin { default: Option<i64>, min: Option<i64>, max: Option<i64> },
    Combo { default: Option<String>, vars: Vec<String> },
    Button,
    String { default: Option<String> },
}

pub enum Requirement {
    Checking,
    Ok,
    Error,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Info {
    Depth(u32),
    SelDepth(u32),
    Time(Duration),
    Nodes(u64),
    Pv(Moves),
    Score(Score),
    Currmove(Move),
    CurrMoveNumber(u32),
    HashFull(u32),
    Nps(u32),
    Thhits(u32),
    Sbhits(u32),
    Cpuload(u32),
    Refutation { mov: Move, line: Moves },
    Currline { cpunr: Option<u32>, line: Moves },
    // Must be at the end so that it appears last
    String(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Score {
    Centipawns { cp: i32, bounds: Option<Bound> },
    Mate { mate: i32 },
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum Bound {
    Lower,
    Upper,
}

impl fmt::Display for UciResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Id { name, author } => {
                writeln!(f, "id name {name}")?;
                write!(f, "id author {author}")
            }
            Self::Uciok => write!(f, "uciok"),
            Self::Readyok => write!(f, "readyok"),
            Self::Bestmove { mov, ponder } => write!(f, "bestmove {mov} {}", Maybe("ponder", ponder)),
            Self::Copyprotection(req) => write!(f, "copyprotection {req}"),
            Self::Registration(req) => write!(f, "registration {req}"),
            Self::Info(info) => {
                let mut info: Vec<&Info> = info.iter().collect();
                info.sort();
                write!(f, "info {}", List(" ", &info))
            }
            Self::Option { name, option } => write!(f, "option name {name} type {option}"),
        }
    }
}

impl fmt::Display for Requirement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ok => write!(f, "ok"),
            Self::Checking => write!(f, "checking"),
            Self::Error => write!(f, "error"),
        }
    }
}

impl fmt::Display for OptionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Check { default } => write!(f, "check {}", Maybe("default", default)),
            Self::Spin { default, min, max } => {
                write!(f, "spin {} {} {}", Maybe("min", min), Maybe("max", max), Maybe("default", default))
            }
            Self::Combo { default, vars } => {
                write!(f, "combo {} {}", Maybe("default", default), List(" ", vars))
            }
            Self::Button => write!(f, "button"),
            Self::String { default } => write!(f, "string {}", Maybe("default", default)),
        }
    }
}

impl fmt::Display for Info {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Depth(depth) => write!(f, "depth {depth}"),
            Self::SelDepth(depth) => write!(f, "seldepth {depth}"),
            Self::Time(time) => write!(f, "time {}", time.as_millis()),
            Self::Nodes(nodes) => write!(f, "nodes {nodes}"),
            Self::Pv(line) => write!(f, "pv {}", List(" ", line)),
            Self::Score(score) => write!(f, "score {score}"),
            Self::Currmove(mov) => write!(f, "currmove {mov}"),
            Self::CurrMoveNumber(num) => write!(f, "currmovenumber {num}"),
            Self::HashFull(permill) => write!(f, "hash {permill}"),
            Self::Nps(nps) => write!(f, "nps {nps}"),
            Self::Thhits(hits) => write!(f, "thhits {hits}"),
            Self::Sbhits(hits) => write!(f, "sbhits {hits}"),
            Self::Cpuload(load) => write!(f, "cpuload {load}"),
            Self::String(str) => write!(f, "str {str}"),
            Self::Refutation { mov, line } => write!(f, "refutation {mov} {}", List(" ", line)),
            Self::Currline { cpunr, line } => write!(f, "{} {}", Maybe("", cpunr), List(" ", line)),
        }
    }
}

impl fmt::Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Centipawns { cp, bounds: Some(Bound::Upper) } => write!(f, "cp {cp} upperbound"),
            Self::Centipawns { cp, bounds: Some(Bound::Lower) } => write!(f, "cp {cp} lowerbound"),
            Self::Centipawns { cp, bounds: None } => write!(f, "cp {cp}"),
            Self::Mate { mate } => write!(f, "mate {mate}"),
        }
    }
}

struct Maybe<'a, T>(&'a str, &'a Option<T>);
impl<T: fmt::Display> fmt::Display for Maybe<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.1 {
            Some(t) => write!(f, "{} {t}", self.0),
            None => Ok(()),
        }
    }
}
struct List<'a, T>(&'a str, &'a [T]);

impl<T> fmt::Display for List<'_, T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Some(first) = self.1.first() else { return Ok(()) };
        write!(f, "{first}")?;
        for val in &self.1[1..] {
            write!(f, "{}{val}", self.0)?;
        }
        Ok(())
    }
}
