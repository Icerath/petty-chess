use crate::{
    engine::transposition::{Nodetype, TranspositionTable},
    prelude::*,
};

impl Board {
    #[must_use]
    pub fn run_perft(&self, depth: u8) -> u64 {
        self.run_perft_with_table(&mut TranspositionTable::default(), depth)
    }

    pub fn run_perft_with_table(&self, table: &mut TranspositionTable<u64>, depth: u8) -> u64 {
        if depth == 0 {
            return 1;
        } else if let Some(entry) = table.get_entry(self, 0, 0, depth) {
            if depth == entry.depth {
                return entry.extra;
            }
        }
        if depth == 1 {
            return self.legal_moves().len() as u64;
        }
        let mut count = 0;
        for mov in self.legal_moves() {
            count += self.with_move(mov).run_perft_with_table(table, depth - 1);
        }
        table.insert(self, &[], depth, 0, Nodetype::Exact, count);
        count
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn perft_start() {
        #[cfg(miri)]
        let results = [1, 20, 400];
        #[cfg(not(miri))]
        let results = [1, 20, 400, 8_902, 197_281, 4_865_609, 119_060_324];

        for (depth, &result) in results.iter().enumerate() {
            let count = Board::start_pos().run_perft(depth as u8);
            assert_eq!(count, result, "depth: {depth}");
        }
    }

    #[test]
    pub fn perft_kiwi() {
        #[cfg(miri)]
        let results = [1, 48];
        #[cfg(not(miri))]
        let results = [1, 48, 2_039, 97_862, 4_085_603, 193_690_690];
        for (depth, &result) in results.iter().enumerate() {
            let count = Board::kiwipete().run_perft(depth as u8);
            assert_eq!(count, result, "depth: {depth}");
        }
    }
    #[test]
    fn perft_position_3() {
        #[cfg(miri)]
        let results = [1, 14, 191];
        #[cfg(not(miri))]
        let results = [1, 14, 191, 2_812, 43_238, 674_624, 11_030_083, 178_633_661];
        for (depth, &result) in results.iter().enumerate() {
            let count = Board::perft_position_3().run_perft(depth as u8);
            assert_eq!(count, result, "depth: {depth}");
        }
    }
    #[test]
    fn perft_position_4() {
        #[cfg(miri)]
        let results = [1, 6, 264];
        #[cfg(not(miri))]
        let results = [1, 6, 264, 9_467, 422_333, 15_833_292 /*, 706_045_033*/];
        for (depth, &result) in results.iter().enumerate() {
            let count = Board::perft_position_4().run_perft(depth as u8);
            assert_eq!(count, result, "depth: {depth}");
        }
    }
    #[test]
    fn perft_talk() {
        #[cfg(miri)]
        let results = [1, 44];
        #[cfg(not(miri))]
        let results = [1, 44, 1_486, 62_379, 2_103_487, 89_941_194];
        for (depth, &result) in results.iter().enumerate() {
            let count = Board::perft_position_5().run_perft(depth as u8);
            assert_eq!(count, result, "depth: {depth}");
        }
    }
}
