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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perft_start() {
        let results = [1, 20, 400, 8_902, 197_281, 4_865_609, 119_060_324];
        for (depth, &result) in results.iter().enumerate() {
            let count = Board::start_pos().run_perft(depth as u8);
            assert_eq!(count, result, "depth: {depth}");
        }
    }

    #[test]
    fn perft_kiwi() {
        let results = [1, 48, 2_039, 97_862, 4_085_603, 193_690_690];
        for (depth, &result) in results.iter().enumerate() {
            let count = Board::kiwipete().run_perft(depth as u8);
            assert_eq!(count, result, "depth: {depth}");
        }
    }
    #[test]
    fn perft_position_3() {
        let results = [1, 14, 191, 2_812, 43_238, 674_624, 11_030_083];
        for (depth, &result) in results.iter().enumerate() {
            let count = Board::perft_position_3().run_perft(depth as u8);
            assert_eq!(count, result, "depth: {depth}");
        }
    }

    #[test]
    fn perft_position_4() {
        let results = [1, 6, 264, 9_467, 422_333, 15_833_292 /*706_045_033*/];
        for (depth, &result) in results.iter().enumerate() {
            let count = Board::perft_position_4().run_perft(depth as u8);
            assert_eq!(count, result, "depth: {depth}");
        }
    }

    #[test]
    fn perft_talk() {
        let results = [1, 44, 1_486, 62_379, 2_103_487];
        for (depth, &result) in results.iter().enumerate() {
            let count = Board::perft_position_5().run_perft(depth as u8);
            assert_eq!(count, result, "depth: {depth}");
        }
    }
}
