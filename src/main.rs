use std::{
    fmt::Write,
    io::BufRead as _,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

use clap::Parser;
use petty_chess::{
    engine::{evaluation::raw_evaluation, transposition::TranspositionTable},
    prelude::*,
    uci::{GoCommand, TimeControl, UciMessage, UciResponse},
};
use tracing::level_filters::LevelFilter;
#[cfg(feature = "tracing")]
use {
    tracing::debug,
    tracing_appender::rolling::{RollingFileAppender, Rotation},
};

#[derive(clap::Parser)]
struct Args {
    #[arg(long = "run")]
    message: Option<String>,
    #[arg(long, default_value = "OFF")]
    log_level: LevelFilter,
}

fn main() {
    let args = Args::parse();

    #[cfg(feature = "tracing")]
    if args.log_level != LevelFilter::OFF {
        let writer = RollingFileAppender::builder()
            .rotation(Rotation::DAILY)
            .filename_suffix("log")
            .build("./logs")
            .unwrap();
        tracing_subscriber::fmt().with_max_level(args.log_level).with_writer(writer).init();
    }
    let mut line = String::new();
    let mut stdin = std::io::stdin().lock();
    let mut app = Application::default();

    if let Some(run) = args.message {
        for section in run.split(';').map(str::trim) {
            if section.is_empty() {
                continue;
            }
            let Some(msg) = UciMessage::parse(section.trim()) else {
                panic!("Invalid message: '{section}'");
            };
            app.process_message(msg, true);
        }
        return;
    }

    while app.running {
        line.clear();
        stdin.read_line(&mut line).unwrap();
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        #[cfg(feature = "tracing")]
        debug!("{line}");

        if let Some(message) = UciMessage::parse(line) {
            app.process_message(message, false);
        } else {
            #[cfg(feature = "tracing")]
            tracing::warn!("Unknown command: '{line}'");
            eprintln!("Unknown command: '{line}'. Type help for more information.");
        }
    }
}

pub struct Application {
    kill: Arc<AtomicBool>,
    engine: Engine,
    running: bool,
    debug: bool,
}

impl Default for Application {
    fn default() -> Self {
        let engine = Engine::new(Board::start_pos());
        Self { kill: engine.kill.clone(), engine, running: true, debug: false }
    }
}

#[expect(clippy::needless_pass_by_value, clippy::unused_self, clippy::match_same_arms)]
impl Application {
    fn process_message(&mut self, msg: UciMessage, wait: bool) {
        use UciMessage as Uci;

        match msg {
            Uci::Uci => self.respond_with_id(),
            Uci::Isready => self.respond(UciResponse::Readyok),
            Uci::Setoption { .. } => {}
            Uci::Debug(on) => self.debug = on,
            Uci::Register(_reg) => {}
            Uci::Ucinewgame => *self = Self::default(),
            Uci::Position { fen, moves } => {
                if let Some(board) = Board::from_fen(&fen) {
                    self.startpos_moves(board, moves);
                } else {
                    #[cfg(feature = "tracing")]
                    tracing::error!("Invalid fen position {fen}");
                }
            }
            Uci::Go(command) if wait => self.go(command).join().unwrap(),
            Uci::Go(command) => _ = self.go(command),
            Uci::Stop => self.kill.store(true, Ordering::Relaxed),
            Uci::PonderHit => {}
            Uci::Quit => self.running = false,
            Uci::Perft { depth } => self.go_perft(depth.unwrap_or(1) as u8),
            Uci::Display => self.display(),
        }
    }

    fn respond_with_id(&self) {
        self.respond(UciResponse::Id {
            name: "Petty Chess".into(),
            author: "Dorje Gilfillan".into(),
        });
        self.respond(UciResponse::Uciok);
    }

    fn respond(&self, response: UciResponse) {
        println!("{response}");
    }

    fn startpos_moves(&mut self, position: Board, moves: Vec<Move>) {
        self.engine.seen_positions = vec![position.zobrist];
        self.engine.board = position;
        for mov in moves {
            let legal_moves = self.engine.board.legal_moves();
            let Some(&mov) = legal_moves.iter().find(|m| {
                (m.from(), m.to(), m.flags().promotion())
                    == (mov.from(), mov.to(), mov.flags().promotion())
            }) else {
                eprintln!("Invalid move: {mov}");
                break;
            };
            if mov.flags().is_capture() {
                self.engine.seen_positions.clear();
            }
            self.engine.seen_positions.push(self.engine.board.zobrist);
            self.engine.board.make_move(mov);
        }
        self.engine.seen_positions.push(self.engine.board.zobrist);
    }

    fn go(&mut self, command: GoCommand) -> JoinHandle<()> {
        self.kill.store(true, Ordering::Relaxed);
        #[cfg(feature = "tracing")]
        let start = Instant::now();
        let time_available = self.get_time_available(command.time_control);
        let mut engine = self.engine.clone();
        engine.kill = Arc::default();
        engine.time_available = time_available;
        self.kill = engine.kill.clone();
        let handle = std::thread::spawn(move || {
            let best_move = engine.search();
            println!("{}", UciResponse::Bestmove { mov: best_move, ponder: None });
            #[cfg(feature = "tracing")]
            {
                tracing::info!("Time taken: {:?}", start.elapsed());
                tracing::info!("Num transpositions: {}", engine.transposition_table.num_hits);
            }
        });
        if time_available != Duration::MAX {
            let kill = self.kill.clone();
            std::thread::spawn(move || {
                std::thread::sleep(time_available);
                kill.store(true, Ordering::Relaxed);
            });
        }
        handle
    }

    fn go_perft(&mut self, depth: u8) {
        let start = Instant::now();
        let total = perft(&self.engine.board, depth);
        eprintln!("\nTime taken: {:?}", start.elapsed());
        eprintln!("Nodes searched: {total}");
    }

    fn get_time_available(&self, time_control: TimeControl) -> Duration {
        match time_control {
            // TODO - ponder
            TimeControl::Ponder => Duration::MAX,
            TimeControl::TimeLeft { wtime, btime, wincr, bincr, .. } => {
                let (total, incr) = if self.engine.board.active_side == White {
                    (wtime, wincr)
                } else {
                    (btime, bincr)
                };
                let estimated_total_moves =
                    i32::from(30.max(self.engine.board.fullmove_counter + 10));
                let moves_to_end =
                    estimated_total_moves - i32::from(self.engine.board.fullmove_counter);
                let time_per_move = total.div_f32(moves_to_end as f32);
                (time_per_move + incr).min(total)
            }
            TimeControl::MoveTime(time) => time,
            TimeControl::Infinite => Duration::MAX,
        }
    }

    fn display(&mut self) {
        let board = &self.engine.board;
        let mut out = String::new();
        for rank in (0..8).rev() {
            out.push_str("+---+---+---+---+---+---+---+---+\n|");
            for file in 0..8 {
                out.push(' ');
                let square =
                    Square::new(Rank::from_int(rank).unwrap(), File::from_int(file).unwrap());
                let piece = board.get_square(square);
                out.push(piece.map_or(' ', Piece::symbol));
                out.push_str(" |");
            }
            _ = writeln!(out, " {}", rank + 1);
        }
        out.push_str("+---+---+---+---+---+---+---+---+\n");
        out.push_str("  a   b   c   d   e   f   g   h  \n");
        println!("{out}");
        println!("Fen: {}", board.to_fen());
        println!("Key: {:?}", board.zobrist);
        print!("Checkers: ");
        board.gen_checkers(board.active_side).for_each(|sq| {
            print!(" {sq}");
        });
        println!();
        println!("Endgame: {:?}", phase(board).endgame().as_float());
        println!("Direct Eval: {:?}", raw_evaluation(board));
    }
}

fn perft(board: &Board, depth: u8) -> u64 {
    let mut total = 0;
    let mut moves = board.legal_moves();

    let mut table = TranspositionTable::default();
    moves.sort();

    for mov in moves {
        let count = board.with_move(mov).run_perft_with_table(&mut table, depth - 1);
        total += count;
        eprintln!("{mov}: {count}");
    }
    total
}
