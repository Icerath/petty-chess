use std::{
    io::BufRead as _,
    time::{Duration, Instant, UNIX_EPOCH},
};

use petty_chess::{
    prelude::*,
    uci::{GoCommand, TimeControl, UciMessage, UciResponse},
};
use tracing::{debug, info, Level};

fn main() -> eyre::Result<()> {
    let duration = UNIX_EPOCH.elapsed().unwrap();
    let _ = std::fs::create_dir("./log");
    let logfile = std::fs::File::create(format!("./log/log-{:x}.log", duration.as_secs()))?;
    tracing_subscriber::fmt().with_max_level(Level::DEBUG).with_ansi(false).with_writer(logfile).init();

    let mut line = String::new();
    let mut stdin = std::io::stdin().lock();
    let mut app = Application::default();
    while app.running {
        line.clear();
        stdin.read_line(&mut line)?;
        let line = line.trim();
        debug!("{line}");

        if let Some(message) = UciMessage::parse(line) {
            app.process_message(message);
        } else {
            tracing::warn!("Unknown command: '{line}'");
            eprintln!("Unknown command: '{line}'. Type help for more information.",);
        }
    }
    Ok(())
}

pub struct Application {
    engine: Engine,
    running: bool,
    debug: bool,
}

impl Default for Application {
    fn default() -> Self {
        Self { engine: Engine::new(Board::start_pos()), running: true, debug: false }
    }
}

#[allow(clippy::needless_pass_by_value, clippy::unused_self, clippy::match_same_arms)]
impl Application {
    fn process_message(&mut self, msg: UciMessage) {
        use UciMessage as Uci;

        match msg {
            Uci::Uci => self.respond_with_id(),
            Uci::Isready => self.respond(UciResponse::Readyok),
            Uci::Setoption { .. } => {}
            Uci::Debug(on) => self.debug = on,
            Uci::Register(_reg) => {}
            Uci::Ucinewgame => {}
            Uci::Position { fen, moves } => {
                if let Some(board) = Board::from_fen(&fen) {
                    self.startpos_moves(board, moves);
                } else {
                    tracing::error!("Invalid fen position {fen}");
                }
            }
            Uci::Go(command) => self.go(command),
            Uci::Stop => self.engine.force_cancelled = true,
            Uci::PonderHit => {}
            Uci::Quit => self.running = false,
            Uci::Perft { depth } => self.go_perft(depth.unwrap_or(1) as u8),
        }
    }
    fn respond_with_id(&self) {
        self.respond(UciResponse::Id { name: "Petty Chess".into(), author: "Dorje Gilfillan".into() });
        self.respond(UciResponse::Uciok);
    }
    fn respond(&self, response: UciResponse) {
        println!("{response}");
    }
    fn startpos_moves(&mut self, position: Board, moves: Moves) {
        self.engine.board = position;
        for mov in moves {
            let legal_moves = self.engine.board.gen_legal_moves();
            let mov = *legal_moves
                .iter()
                .find(|m| {
                    (m.from(), m.to(), m.flags().promotion())
                        == (mov.from(), mov.to(), mov.flags().promotion())
                })
                .unwrap();
            self.engine.board.make_move(mov);
            eprintln!("Seen: {}", self.engine.board.seen_position());
        }
        eprintln!("Direct eval at pos: {}", self.engine.raw_evaluation());
    }
    fn go(&mut self, command: GoCommand) {
        self.set_time_available(command.time_control);
        let best_move = self.engine.search();

        info!(
            best_move = best_move.to_string(),
            depth_reached = self.engine.depth_reached,
            effective_nodes = self.engine.effective_nodes,
            total_nodes = self.engine.total_nodes,
            time_taken = tracing::field::debug(self.engine.time_started.elapsed()),
        );
        self.respond(UciResponse::Bestmove { mov: best_move, ponder: None });
    }
    fn go_perft(&mut self, depth: u8) {
        let start = Instant::now();
        let total = perft(&mut self.engine.board, depth, true);
        eprintln!("\nTime taken: {:?}", start.elapsed());
        eprintln!("Nodes searched: {total}");
    }
    fn set_time_available(&mut self, time_control: TimeControl) {
        match time_control {
            // TODO - ponder
            TimeControl::Ponder => self.engine.time_available = Duration::MAX,
            TimeControl::TimeLeft { wtime, btime, wincr, bincr, .. } => {
                let (total, incr) =
                    if self.engine.board.white_to_play() { (wtime, wincr) } else { (btime, bincr) };
                let estimated_total_moves = i32::from(30.max(self.engine.board.fullmove_counter + 10));
                let moves_to_end = estimated_total_moves - i32::from(self.engine.board.fullmove_counter);
                let time_per_move = total.div_f32(moves_to_end as f32);
                self.engine.time_available = (time_per_move + incr.mul_f32(0.9)).min(total);
            }
            TimeControl::MoveTime(time) => self.engine.time_available = time,
            TimeControl::Infinite => self.engine.time_available = Duration::MAX,
        }
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
            eprintln!("{mov}: {count}");
        }
    }
    total
}
