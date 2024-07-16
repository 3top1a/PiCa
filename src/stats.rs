use crate::utils::MAX_MOVES;

pub static mut NODES_SEARCHED: u64 = 0;
pub static mut QNODES_SEARCHED: u64 = 0;
pub static mut CHECK_EXTENSION: i32 = 0;
pub static mut TT_CHECK: i32 = 0;
pub static mut TT_HIT: i32 = 0;

pub static mut MOVE_INDEX_DIST: [u32; MAX_MOVES] = [0; MAX_MOVES];

#[macro_export]
macro_rules! bump {
    ($var:expr) => {
        unsafe {
            $var += 1;
        }
    };
}

pub fn reset() {
    unsafe {
        QNODES_SEARCHED = 0;
        NODES_SEARCHED = 0;
        CHECK_EXTENSION = 0;
        TT_CHECK = 0;
        TT_HIT = 0;
        MOVE_INDEX_DIST = [0; MAX_MOVES];
    };
}

pub fn add_move_index(i: usize) {
    unsafe { MOVE_INDEX_DIST [i] += 1 }
}
