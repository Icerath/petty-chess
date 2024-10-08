use std::{
    fmt::Write,
    io::BufRead as _,
    time::{Duration, Instant},
};

use petty_chess::{
    engine::transposition::TranspositionTable,
    prelude::*,
    uci::{GoCommand, TimeControl, UciMessage, UciResponse},
};
#[cfg(feature = "tracing")]
use tracing::{debug, Level};
#[cfg(feature = "tracing")]
use tracing_appender::rolling::{RollingFileAppender, Rotation};

fn main() {
    #[cfg(feature = "tracing")]
    let writer = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_suffix("log")
        .build("./logs")
        .unwrap();

    #[cfg(feature = "tracing")]
    tracing_subscriber::fmt().with_max_level(Level::DEBUG).with_writer(writer).init();
    let mut line = String::new();
    let mut stdin = std::io::stdin().lock();
    let mut app = Application::default();
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
            app.process_message(message);
        } else {
            #[cfg(feature = "tracing")]
            tracing::warn!("Unknown command: '{line}'");
            eprintln!("Unknown command: '{line}'. Type help for more information.",);
        }
    }
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
            Uci::Ucinewgame => *self = Self::default(),
            Uci::Position { fen, moves } => {
                if let Some(board) = Board::from_fen(&fen) {
                    self.startpos_moves(board, moves);
                } else {
                    #[cfg(feature = "tracing")]
                    tracing::error!("Invalid fen position {fen}");
                }
            }
            Uci::Go(command) => self.go(command),
            Uci::Stop => self.engine.force_cancelled = true,
            Uci::PonderHit => {}
            Uci::Quit => self.running = false,
            Uci::Perft { depth } => self.go_perft(depth.unwrap_or(1) as u8),
            Uci::Display => self.display(),
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
        self.engine.seen_positions = vec![position.zobrist];
        self.engine.board = position.clone();
        for mov in moves {
            let legal_moves = self.engine.board.gen_legal_moves();
            let Some(&mov) = legal_moves.iter().find(|m| {
                (m.from(), m.to(), m.flags().promotion()) == (mov.from(), mov.to(), mov.flags().promotion())
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
    fn go(&mut self, command: GoCommand) {
        #[cfg(feature = "tracing")]
        let start = Instant::now();
        self.set_time_available(command.time_control);
        let best_move = self.engine.search();
        self.respond(UciResponse::Bestmove { mov: best_move, ponder: None });
        #[cfg(feature = "tracing")]
        tracing::info!("Time taken: {:?}", start.elapsed());
        #[cfg(feature = "tracing")]
        tracing::info!("Num transpositions: {}", self.engine.transposition_table.num_hits);
    }
    fn go_perft(&mut self, depth: u8) {
        let start = Instant::now();
        let total = perft(&mut self.engine.board, depth);
        eprintln!("\nTime taken: {:?}", start.elapsed());
        eprintln!("Nodes searched: {total}");
    }
    fn set_time_available(&mut self, time_control: TimeControl) {
        match time_control {
            // TODO - ponder
            TimeControl::Ponder => self.engine.time_available = Duration::MAX,
            TimeControl::TimeLeft { wtime, btime, wincr, bincr, .. } => {
                let (total, incr) =
                    if self.engine.board.active_side == White { (wtime, wincr) } else { (btime, bincr) };
                let estimated_total_moves = i32::from(30.max(self.engine.board.fullmove_counter + 10));
                let moves_to_end = estimated_total_moves - i32::from(self.engine.board.fullmove_counter);
                let time_per_move = total.div_f32(moves_to_end as f32);
                self.engine.time_available = (time_per_move + incr).min(total);
            }
            TimeControl::MoveTime(time) => self.engine.time_available = time,
            TimeControl::Infinite => self.engine.time_available = Duration::MAX,
        }
    }
    fn display(&mut self) {
        let mut out = String::new();
        for rank in (0..8).rev() {
            out.push_str("+---+---+---+---+---+---+---+---+\n|");
            for file in 0..8 {
                out.push(' ');
                let square = Square::new(Rank(rank), File(file));
                let piece = self.engine.board.get_square(square);
                out.push(piece.map_or(' ', Piece::symbol));
                out.push_str(" |");
            }
            _ = writeln!(out, " {}", rank + 1);
        }
        out.push_str("+---+---+---+---+---+---+---+---+\n");
        out.push_str("  a   b   c   d   e   f   g   h  \n");
        println!("{out}");
        println!("Fen: {}", self.engine.board.to_fen());
        println!("Key: {:?}", self.engine.board.zobrist);
        print!("Checkers: ");
        self.engine.board.checkers.for_each(|sq| {
            print!(" {sq}");
        });
        println!();
        println!("Direct Eval: {:?}", self.engine.raw_evaluation());
    }
}

fn perft(board: &mut Board, depth: u8) -> u64 {
    let mut total = 0;
    let mut moves = board.gen_legal_moves();

    let mut table = TranspositionTable::default();
    moves.sort_by_key(|mov| mov.from().int() + mov.to().int());

    for mov in moves {
        let unmake = board.make_move(mov);
        let count = board.run_perft_with_table(&mut table, depth - 1);
        total += count;
        board.unmake_move(unmake);
        eprintln!("{mov}: {count}");
    }
    total
}
