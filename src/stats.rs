pub static mut NODES_SEARCHED: i32 = 0;
pub static mut CHECK_EXTENSION: i32 = 0;

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
    };
}

pub fn dump(depth: i32) {
    unsafe {
        println!("info {NODES_SEARCHED}");
        println!("stats checkexts {} EBR {}", CHECK_EXTENSION, (NODES_SEARCHED as f32).powf(1. / depth as f32));
    }
}
