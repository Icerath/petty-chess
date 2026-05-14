use std::{
    fmt::Write,
    io::BufRead as _,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
    },
    time::{Duration, Instant},
};

use petty_chess::{
    engine::{evaluation::raw_evaluation, transposition::TranspositionTable},
    prelude::*,
    uci::{GoCommand, TimeControl, UciMessage, UciResponse},
};

fn main() {
    let mut line = String::new();
    let mut stdin = std::io::stdin().lock();

    let (tx, rx) = std::sync::mpsc::channel();

    let mut app = Application::new(tx);

    std::thread::spawn(move || {
        for (mut engine, command) in rx {
            engine.kill.store(false, Ordering::Release);
            engine.time_available = get_time_available(&engine.board, command.time_control);
            engine.search();
        }
    });

    while app.running {
        line.clear();
        stdin.read_line(&mut line).unwrap();
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(message) = UciMessage::parse(line) {
            app.process_message(message);
        } else {
            eprintln!("Unknown command: '{line}'. Type help for more information.");
        }
    }
}

pub struct Application {
    tx: Sender<(Engine, GoCommand)>,
    kill: Arc<AtomicBool>,
    engine: Engine,
    running: bool,
    debug: bool,
}

#[expect(clippy::needless_pass_by_value, clippy::unused_self, clippy::match_same_arms)]
impl Application {
    fn new(tx: Sender<(Engine, GoCommand)>) -> Self {
        let engine = Engine::new(Board::start_pos());
        Self { kill: engine.kill.clone(), engine, running: true, debug: false, tx }
    }

    fn process_message(&mut self, msg: UciMessage) {
        use UciMessage as Uci;

        match msg {
            Uci::Uci => self.respond_with_id(),
            Uci::Isready => self.respond(UciResponse::Readyok),
            Uci::Setoption { .. } => {}
            Uci::Debug(on) => self.debug = on,
            Uci::Register(_reg) => {}
            Uci::Ucinewgame => *self = Self::new(self.tx.clone()),
            Uci::Position { fen, moves } => {
                if let Some(board) = Board::from_fen(&fen) {
                    self.startpos_moves(board, moves);
                }
            }
            Uci::Go(command) => {
                _ = self.tx.send((self.engine.clone(), command));
            }

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

    fn go_perft(&mut self, depth: u8) {
        let start = Instant::now();
        let total = perft(&mut self.engine.board, depth);
        eprintln!("\nTime taken: {:?}", start.elapsed());
        eprintln!("Nodes searched: {total}");
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
        for sq in board.gen_checkers(board.active_side) {
            print!(" {sq}");
        }
        println!();
        println!("Endgame: {:?}", phase(board).endgame().as_float());
        println!("Direct Eval: {:?}", raw_evaluation(board));
    }
}

fn perft(board: &mut Board, depth: u8) -> u64 {
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

#[expect(clippy::match_same_arms)]
fn get_time_available(board: &Board, time_control: TimeControl) -> Duration {
    match time_control {
        // TODO - ponder
        TimeControl::Ponder => Duration::MAX,
        TimeControl::TimeLeft { wtime, btime, wincr, bincr, .. } => {
            let (total, incr) =
                if board.active_side == White { (wtime, wincr) } else { (btime, bincr) };
            let estimated_total_moves = 30.max(board.halfmove_clock as i32 * 2 + 10);
            let moves_to_end = estimated_total_moves - board.halfmove_clock as i32 * 2;
            let time_per_move = total.div_f32(moves_to_end as f32);
            (time_per_move + incr).min(total)
        }
        TimeControl::MoveTime(time) => time,
        TimeControl::Infinite => Duration::MAX,
    }
}
