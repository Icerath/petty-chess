use std::io::{stdin, BufRead};

use eyre::ContextCompat;
use petty_chess::prelude::*;
use tracing::info;

fn main() -> eyre::Result<()> {
    let file_appender = tracing_appender::rolling::hourly("./log", "prefix.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt().with_ansi(false).with_writer(non_blocking).init();

    let mut stdin = stdin().lock();
    let mut line = String::new();
    let mut engine = Application::default();
    loop {
        line.clear();
        stdin.read_line(&mut line)?;
        let exit = engine.process_line(line.trim())?;
        if exit {
            break;
        }
    }
    Ok(())
}

pub struct Application {
    engine: Engine,
}

impl Default for Application {
    fn default() -> Self {
        Self { engine: Engine::new(Board::start_pos()) }
    }
}

impl Application {
    fn process_line(&mut self, line: &str) -> eyre::Result<bool> {
        info!("{line}");
        match line {
            "quit" => return Ok(true),
            "uci" => println!("uciok"),
            "ucinewgame" => {}
            "isready" => println!("readyok"),
            "position startpos" => self.engine.board = Board::start_pos(),
            _ if line.starts_with("position fen") => {
                self.fen_position(line.trim_start_matches("position fen").trim())?;
            }
            _ if line.starts_with("position startpos moves") => {
                self.startpos_moves(line.trim_start_matches("position startpos moves"))?;
            }
            _ if line.starts_with("go perft") => self.go_perft(line.trim_start_matches("go perft")),
            _ if line.starts_with("go") => self.go(line.trim_start_matches("go")),
            _ => {}
        }

        Ok(false)
    }
    fn startpos_moves(&mut self, input: &str) -> eyre::Result<()> {
        let moves = input
            .split_whitespace()
            .map(|mov| {
                let from: Pos = mov[..2].parse()?;
                let to: Pos = mov[2..4].parse()?;
                let promote: Option<Promotion> = mov[4..].parse().ok();
                Ok((from, to, promote))
            })
            .collect::<eyre::Result<Vec<_>>>()?;

        self.engine.board = Board::start_pos();
        for (from, to, promote) in moves {
            let legal_moves = self.engine.board.gen_legal_moves();
            let mov = *legal_moves
                .iter()
                .find(|mov| mov.from() == from && mov.to() == to && mov.flags().promotion() == promote)
                .unwrap();
            self.engine.board.make_move(mov);
        }

        Ok(())
    }
    fn fen_position(&mut self, fen: &str) -> eyre::Result<()> {
        self.engine.board = Board::from_fen(fen).wrap_err("InvalidFen")?;
        Ok(())
    }

    fn go(&mut self, _command: &str) {
        let best_move = self.engine.search();
        info!(
            "Move={}, Depth={}, Nodes={}, TotalNodes={}",
            best_move,
            self.engine.depth_reached,
            self.engine.nodes_evaluated_for_heighest_depth,
            self.engine.nodes_evaluated,
        );

        println!("bestmove {best_move}");
    }
    fn go_perft(&mut self, rest: &str) {
        let depth = rest.trim().parse().unwrap_or(1);
        let total = perft(&mut self.engine.board, depth, true);

        println!("\nNodes searched: {total}");
    }
}

fn perft(board: &mut Board, depthleft: u8, print: bool) -> u64 {
    if depthleft == 0 {
        return 1;
    } else if depthleft == 1 && !print {
        return board.gen_legal_moves().len() as u64;
    }
    let mut total = 0;
    let mut moves = board.gen_legal_moves();
    if print {
        moves.sort_by_key(|mov| mov.from().0 + mov.to().0);
    }

    for mov in moves {
        let unmake = board.make_move(mov);
        let count = perft(board, depthleft - 1, false);
        total += count;
        board.unmake_move(unmake);

        if print {
            println!("{mov}: {count}");
        }
    }
    total
}
