use crate::{
    engine::transposition::{Nodetype, TranspositionTable},
    prelude::*,
};

impl Board {
    pub fn run_perft(&mut self, depth: u8) -> u64 {
        self.perft(&mut TranspositionTable::default(), depth)
    }

    fn perft(&mut self, table: &mut TranspositionTable, depth: u8) -> u64 {
        if depth == 0 {
            return 1;
        } else if let Some(entry) = table.get_entry(self, 0, 0, depth) {
            if depth == entry.depth {
                return entry.treesize;
            }
        }
        if depth == 1 {
            return self.gen_legal_moves().len() as u64;
        }
        let mut count = 0;
        for mov in self.gen_legal_moves() {
            let unmake = self.make_move(mov);
            count += self.perft(table, depth - 1);
            self.unmake_move(unmake);
        }
        table.insert(self, depth, 0, Nodetype::Exact, count);
        count
    }
}
