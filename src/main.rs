#![no_main]
#![no_std]

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use heapless::String;
use core::fmt::Write;

use stm32l4xx_hal as _;
use panic_halt as _;

mod db;
mod types;
mod flash;

use db::{Database, MAX_RECORDS};
use types::{Key, Value, Record};

#[entry]
fn main() -> ! {
    let mut db = Database::new();

    // Construct key and value
    let mut key: Key = String::new();
    key.push_str("temp").unwrap();

    let mut value: Value = String::new();
    value.push_str("24.5C").unwrap();

    // Create and persist
    let _ = db.create(key.clone(), value.clone());
    let record = Record { key: key.clone(), value: value.clone() };
    let _ = db.persist(&record);

    // Simulate "reboot" (just to show restore again)
    let mut db = Database::new(); // simulates power cycle
    db.restore();

    if let Some(v) = db.read(&key) {
        let _ = hprintln!("Restored value: {}", v);
    } else {
        let _ = hprintln!("Key not found after restore");
    }

    for i in 0..(MAX_RECORDS + 2) { // try writing more than max
        let mut key = String::new();
        let _ = write!(key, "key{}", i);
        let mut value = String::new();
        let _ = write!(value, "val{}", i);
        let record = Record { key: key.clone(), value: value.clone() };
        
        let create_res = db.create(key.clone(), value.clone());
        let persist_res = db.persist(&record);
        
        if create_res.is_err() || persist_res.is_err() {
            let _ = hprintln!("DB full at record {}", i);
            break;
        }
    }

    loop {}
}
