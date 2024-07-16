use std::ptr::{addr_of, addr_of_mut};

use chess::Board;
use engine::Engine;
use pica::*;
use time::TimeManager;
use tuning::{score, MOVE_INDEX_DIST};
use utils::{History, HASH_VALUE, KILLER_VALUE, PV_VALUE};

fn main() {
    let mut eng = Engine::new(128);
    eng.start(Board::default(), TimeManager::test_preset(), History::new());

    // Tune move ord
    // println!("{:#?}", unsafe { MOVE_INDEX_DIST.clone() } );

    let mut best_score = unsafe { stats::NODES_SEARCHED };
    println!("Score: {}", best_score);
    println!("Sum: {}", unsafe { MOVE_INDEX_DIST.clone().len() });

    let mutables = unsafe {
        [
            ("PV", addr_of_mut!(PV_VALUE)),
            ("HASH", addr_of_mut!(HASH_VALUE)),
            ("KILLER", addr_of_mut!(KILLER_VALUE)),
        ]
    };

    loop {
        let increment = fastrand::u32(..) as i32 % 20;

        let mutable_entry = fastrand::choice(mutables).unwrap();
        let mutable = mutable_entry.1;
        unsafe {
            mutable.write_unaligned(mutable.read_unaligned().wrapping_add(increment as u32));
        }

        // reset
        unsafe {
            MOVE_INDEX_DIST = vec![];
            stats::reset();
        }
        let mut eng = Engine::new(128);
        eng.start(Board::default(), TimeManager::test_preset(), History::new());

        let curr_score = unsafe { stats::NODES_SEARCHED };
        if curr_score < best_score {
            best_score = curr_score;
            println!("Score: {}", best_score);
            println!("Changed {} to {}", mutable_entry.0, unsafe {
                mutable.read_unaligned()
            });
        } else {
            unsafe {
                mutable.write_unaligned(mutable.read_unaligned().wrapping_sub(increment as u32));
            }
        }
    }
}
