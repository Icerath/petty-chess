use std::{str::FromStr, time::Duration};

use super::{GoCommand, Registration, TimeControl, UciMessage as Uci};
use crate::prelude::*;

impl FromStr for Uci {
    type Err = ();
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::parse(input).ok_or(())
    }
}

impl Uci {
    pub fn parse(input: &str) -> Option<Self> {
        let mut tokens = Lexer::new(input);

        loop {
            return match tokens.bump()?.as_str() {
                "uci" => Some(Uci::Uci),
                "debug" => Some(match tokens.bump()?.as_str() {
                    "on" => Uci::Debug(true),
                    "off" => Uci::Debug(false),
                    _ => continue,
                }),
                "isready" => Some(Uci::Isready),
                "setoption" => {
                    let "name" = tokens.bump()?.as_str() else { continue };
                    let id = tokens.bump()?;
                    let mut value = None;
                    if tokens.bump() == Some("value".to_string()) {
                        value = Some(tokens.bump()?);
                    }
                    Some(Uci::Setoption { id, value })
                }
                "register" => {
                    let mut name = None;
                    let mut code = None;
                    loop {
                        let Some(token) = tokens.bump() else {
                            break Some(Uci::Register(Registration::Now { name, code }));
                        };
                        match token.as_str() {
                            "later" => break Some(Uci::Register(Registration::Later)),
                            "name" => name = tokens.bump().map(String::from),
                            "code" => code = tokens.bump().map(String::from),
                            _ => continue,
                        }
                    }
                }
                "ucinewgame" => Some(Uci::Ucinewgame),
                "position" => {
                    let fen = String::from(match tokens.bump()?.as_str() {
                        "startpos" => fen::STARTING_FEN,
                        "kiwipete" => fen::KIWIPETE,
                        "fen" => tokens.fen()?,
                        _ => continue,
                    });
                    Some(Uci::Position { fen, moves: tokens.moves() })
                }
                "go" => Some(Uci::Go(Self::parse_go(&mut tokens))),
                "stop" => Some(Uci::Stop),
                "ponderhit" => Some(Uci::PonderHit),
                "quit" => Some(Uci::Quit),
                "perft" => Some(Uci::Perft { depth: tokens.bump_spin().map(|i| i as u32) }),
                "d" => Some(Uci::Display),
                _ => continue,
            };
        }
    }
    fn parse_go(tokens: &mut Lexer) -> GoCommand {
        let mut command = GoCommand {
            searchmoves: None,
            time_control: TimeControl::Infinite,
            depth: None,
            nodes: None,
            mate: None,
        };
        let mut wtime = Duration::ZERO;
        let mut btime = Duration::ZERO;
        let mut wincr = Duration::ZERO;
        let mut bincr = Duration::ZERO;
        let mut moves_to_go = None;
        loop {
            let Some(token) = tokens.bump() else { break };
            match token.as_str() {
                "infinite" => command.time_control = TimeControl::Infinite,
                "ponder" => command.time_control = TimeControl::Ponder,
                "movetime" => {
                    let Some(movetime) = tokens.bump_spin() else { continue };
                    command.time_control = TimeControl::MoveTime(Duration::from_millis(movetime));
                }

                token @ ("wtime" | "btime" | "wincr" | "bincr" | "movestogo") => {
                    let Some(next) = tokens.bump_spin() else { continue };
                    match token {
                        "wtime" => wtime = Duration::from_millis(next),
                        "btime" => btime = Duration::from_millis(next),
                        "wincr" => wincr = Duration::from_millis(next),
                        "bincr" => bincr = Duration::from_millis(next),
                        "movestogo" => moves_to_go = Some(next as u32),
                        _ => unreachable!(),
                    }
                    command.time_control = TimeControl::TimeLeft { wtime, btime, wincr, bincr, moves_to_go }
                }
                "searchmoves" => command.searchmoves = Some(tokens.moves()),
                "mate" => command.mate = tokens.bump_spin().map(|i| i as u32).or(command.mate),
                "nodes" => command.nodes = tokens.bump_spin().or(command.nodes),
                "depth" => command.depth = tokens.bump_spin().map(|i| i as u32).or(command.depth),
                _ => continue,
            }
        }
        command
    }
}

#[derive(Debug, Clone)]
struct Lexer<'a> {
    remaining: &'a str,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Self { remaining: input.trim() }
    }
    fn bump(&mut self) -> Option<String> {
        if self.remaining.is_empty() {
            return None;
        }
        let ws = self.remaining.find(|c: char| c.is_whitespace()).unwrap_or(self.remaining.len());
        let token = &self.remaining[..ws];
        self.remaining = self.remaining[ws..].trim();
        Some(token.to_ascii_lowercase())
    }
    fn bump_spin(&mut self) -> Option<u64> {
        if self.remaining.is_empty() {
            return None;
        }
        let end = self.remaining.find(|c: char| !c.is_ascii_digit()).unwrap_or(self.remaining.len());
        let token = &self.remaining[..end];
        self.remaining = self.remaining[end..].trim();
        token.parse().ok()
    }
    fn moves(&mut self) -> Moves {
        let mut moves = Moves::new();
        loop {
            let Some(token) = self.bump() else { break };
            let Ok(mov) = token.parse() else { continue };
            moves.push(mov);
        }
        moves
    }
    fn peek(&self) -> Option<String> {
        self.clone().bump()
    }
    fn fen(&mut self) -> Option<&'a str> {
        let start = self.remaining;

        self.bump()?;
        self.bump()?;
        self.bump()?;
        self.bump()?;

        for _ in 0..2 {
            if self.peek().is_some_and(|token| u32::from_str(&token).is_ok()) {
                self.bump().expect("Peek returning some should guarantee bump succeeds");
            }
        }
        let length = start.len() - self.remaining.len();
        Some(&start[..length])
    }
}

#[test]
fn test_uci_parsing() {
    assert_eq!(
        "asdhasud    poSition   StarTpos 123124  213y1279asdzxc".parse(),
        Ok(Uci::Position { fen: fen::STARTING_FEN.into(), moves: Moves::new() })
    );
    assert_eq!(
        "position fen 8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -".parse(),
        Ok(Uci::Position { fen: fen::PERFT_POSITION_3.into(), moves: Moves::new() })
    );

    assert_eq!("hgfgfas debug on garbage".parse(), Ok(Uci::Debug(true)));
    assert_eq!("debug off garbage".parse(), Ok(Uci::Debug(false)));

    assert_eq!("perft    garbage".parse(), Ok(Uci::Perft { depth: None }));
    assert_eq!("perft    4 ".parse(), Ok(Uci::Perft { depth: Some(4) }));
    assert_eq!("perft    8a".parse(), Ok(Uci::Perft { depth: Some(8) }));

    assert_eq!("1283698 go snkdmzx9".parse(), Ok(Uci::Go(GoCommand::default())));
    assert_eq!(
        "go depth 4 nodes 5 movestogo 6 movetime 10 mate 4".parse(),
        Ok(Uci::Go(GoCommand {
            depth: Some(4),
            nodes: Some(5),
            mate: Some(4),
            searchmoves: None,
            time_control: TimeControl::MoveTime(Duration::from_millis(10))
        }))
    );
    assert_eq!(
        "go depth searchmoves e2e4q e7e5".parse(),
        Ok(Uci::Go(GoCommand {
            searchmoves: Some(
                vec![
                    Move::new(Square::E2, Square::E4, MoveFlags::QueenPromotion),
                    Move::new(Square::E7, Square::E5, MoveFlags::Quiet)
                ]
                .into()
            ),
            ..GoCommand::default()
        }))
    );
    assert_eq!(
        "go depth wtime 10000 btime 10000 wincr 5000bincr 3000movestogo 5".parse(),
        Ok(Uci::Go(GoCommand {
            time_control: TimeControl::TimeLeft {
                wtime: Duration::from_millis(10000),
                btime: Duration::from_millis(10000),
                wincr: Duration::from_millis(5000),
                bincr: Duration::from_millis(3000),
                moves_to_go: Some(5),
            },
            ..GoCommand::default()
        }))
    );
}
