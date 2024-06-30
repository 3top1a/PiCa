pub static mut NODES_SEARCHED: i32 = 0;
pub static mut CHECK_EXTENSION: i32 = 0;

pub static mut TT_CHECK: i32 = 0;
pub static mut TT_HIT: i32 = 0;

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
        NODES_SEARCHED = 0;
        CHECK_EXTENSION = 0;
        TT_CHECK = 0;
        TT_HIT = 0;
    };
}
