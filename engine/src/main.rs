use std::io::{stdin, BufRead};

use engine::prelude::*;

pub fn main() -> eyre::Result<()> {
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
        match line {
            "quit" => return Ok(true),
            "uci" => println!("uciok"),
            "ucinewgame" => {}
            "isready" => println!("readyok"),
            "position startpos" => self.engine.board = Board::start_pos(),
            _ if line.starts_with("position startpos moves") => {
                self.startpos_moves(line.trim_start_matches("position startpos moves"))?;
            }
            _ if line.starts_with("go") => self.process_go_command(line.trim_start_matches("go "))?,
            _ => {}
        }

        Ok(false)
    }
    fn startpos_moves(&mut self, input: &str) -> eyre::Result<()> {
        let moves = input
            .split_whitespace()
            .map(|mov| {
                let from: Pos = mov[..2].parse().unwrap();
                let to: Pos = mov[2..4].parse().unwrap();
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
    fn process_go_command(&mut self, _command: &str) -> eyre::Result<()> {
        let best_move = self.engine.search();
        println!("bestmove {best_move}");
        Ok(())
    }
}
