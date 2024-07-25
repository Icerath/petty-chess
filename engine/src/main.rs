use std::io::{stdin, BufRead};

use engine::{ai::Engine, prelude::Board};

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
            _ if line.starts_with("go") => self.process_go_command(line.trim_start_matches("go "))?,
            _ => {}
        }

        Ok(false)
    }
    fn process_go_command(&mut self, _command: &str) -> eyre::Result<()> {
        let best_move = self.engine.search();
        println!("bestmove {best_move}");
        Ok(())
    }
}
