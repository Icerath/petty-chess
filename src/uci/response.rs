use std::{fmt, time::Duration};

use crate::prelude::*;

pub enum UciResponse {
    Id { name: String, author: String },
    Uciok,
    Readyok,
    Bestmove { mov: Move, ponder: Option<Move> },
    Copyprotection(Requirement),
    Registration(Requirement),
    Info(Box<Info>),
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

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Info {
    pub depth: Option<u32>,
    pub seldepth: Option<u32>,
    pub time: Option<Duration>,
    pub nodes: Option<u64>,
    pub pv: Option<Moves>,
    pub score: Option<Score>,
    pub currmove: Option<Move>,
    pub currmovnum: Option<u32>,
    pub hash_full: Option<u32>,
    pub nps: Option<u32>,
    pub thhits: Option<u32>,
    pub sbhits: Option<u32>,
    pub cpu_load: Option<u32>,
    pub string: Option<String>,
    pub refutation: Option<(Move, Moves)>,
    pub currline: Option<(Option<u32>, Moves)>,
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
            Self::Info(info) => write!(f, "info{info}"),
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
        write!(f, "{}", Maybe(" depth", &self.depth))?;
        write!(f, "{}", Maybe(" seldepth", &self.seldepth))?;
        write!(f, "{}", Maybe(" score", &self.score))?;
        write!(f, "{}", Maybe(" nodes", &self.nodes))?;
        write!(f, "{}", Maybe(" nps", &self.nps))?;
        write!(f, "{}", Maybe(" hashfull", &self.hash_full))?;
        write!(f, "{}", Maybe(" tbhits", &self.thhits))?;
        write!(f, "{}", Maybe(" sbhits", &self.sbhits))?;
        write!(f, "{}", Maybe(" cpuload", &self.cpu_load))?;
        write!(f, "{}", Maybe(" time", &self.time.map(|time| time.as_millis())))?;
        write!(f, "{}", Maybe(" currmove", &self.currmove))?;
        write!(f, "{}", Maybe(" currmovenumber", &self.currmovnum))?;
        write!(f, "{}", Maybe(" pv", &self.pv.as_ref().map(|line| List(" ", line))))?;

        if let Some((mov, line)) = &self.refutation {
            write!(f, " refutation {mov} {}", List(" ", line))?;
        }
        if let Some((cpunr, line)) = &self.currline {
            write!(f, " {} {}", Maybe("", cpunr), List(" ", line))?;
        }

        write!(f, "{}", Maybe(" str", &self.string))
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
