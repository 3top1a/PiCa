use chess::{CacheTable, ChessMove};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Debug)]
pub enum NodeType {
    Exact,
    LowerBound,
    UpperBound,
    /// Default entry for invalid/unused entries
    Default,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Debug)]
/// Transposition table entry
/// TODO: Compressed TT entry in 128 bits - 64 for key, 64 for data. Might have to cut the move and reimplement TT move ordering
pub struct TranspositionEntry {
    pub key: u64,
    pub depth: u8,
    pub node_type: NodeType,
    pub value: i32,
    pub best_move: Option<ChessMove>,
}

impl Default for TranspositionEntry {
    fn default() -> Self {
        Self {
            depth: 0,
            key: 0,
            node_type: NodeType::Default,
            value: 0,
            best_move: None,
        }
    }
}

impl TranspositionEntry {
    pub fn is_valid(&self, key: u64) -> bool {
        // return self.key == key && *self != Self::default()
        self.key == key && self.node_type != NodeType::Default
    }
}

pub struct TT {
    // TODO maybe get rid of CacheTable in favor of custom impl
    t: CacheTable<TranspositionEntry>,
}

impl TT {
    /// Create a new Transposition Table with a said size in MiB
    pub fn new_with_size_mb(mb: usize) -> Self {
        let n_entries = mb * 1_048_576 / 16;

        Self {
            t: CacheTable::new(n_entries, TranspositionEntry::default()),
        }
    }

    pub fn set(&mut self, a: TranspositionEntry) {
        self.t.add(a.key, a);
    }

    pub fn get(&self, hash: u64) -> TranspositionEntry {
        self.t.get(hash).unwrap_or_default()
    }
}

mod test {
    #[test]
    fn test_tt() {
        use super::TranspositionEntry;
        use super::TT;
        // No matter what, a default TE is not valid
        let default = TranspositionEntry::default();
        assert!(!default.is_valid(0));
        assert!(!default.is_valid(1));

        let a = TranspositionEntry {
            depth: 2,
            key: 85,
            node_type: crate::tt::NodeType::Exact,
            value: 0,
            ..Default::default()
        };

        assert!(a.is_valid(85));
        assert!(!a.is_valid(86));

        let mut t = TT::new_with_size_mb(32);
        assert!(!t.get(a.key).is_valid(a.key));
        t.set(a);

        assert_eq!(t.get(a.key), a);
        assert_eq!(t.get(a.key).best_move, a.best_move);
        assert!(t.get(a.key).is_valid(a.key));
    }
}
