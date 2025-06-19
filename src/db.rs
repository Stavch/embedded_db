use crate::types::{Key, Value, Record, RECORD_SIZE};
use crate::flash::{FlashWriter, DB_START, DB_SIZE};

use heapless::FnvIndexMap;
use cortex_m_semihosting::hprintln;

#[cfg(feature = "simulate_constraints")]
pub const MAX_RECORDS: usize = 4;

#[cfg(not(feature = "simulate_constraints"))]
pub const MAX_RECORDS: usize = 16;

pub struct Database {
    store: FnvIndexMap<Key, Value, MAX_RECORDS>,
    next_offset: usize,
}

impl Database {
    pub fn new() -> Self {
        let mut db = Self {
            store: FnvIndexMap::new(),
            next_offset: 0,
        };
        db.restore();
        db
    }

    pub fn create(&mut self, key: Key, value: Value) -> Result<(), &'static str> {
        self.store.insert(key, value).map_err(|_| {
            #[cfg(feature = "simulate_constraints")]
            hprintln!("💡 Simulated RAM limit hit: MAX_RECORDS reached");
            "DB full"
        })?;
        Ok(())
    }

    pub fn read(&self, key: &Key) -> Option<&Value> {
        self.store.get(key)
    }

    pub fn update(&mut self, key: &Key, value: Value) -> Result<(), &'static str> {
        if let Some(v) = self.store.get_mut(key) {
            *v = value;
            Ok(())
        } else {
            Err("Key not found")
        }
    }

    pub fn delete(&mut self, key: &Key) -> Result<(), &'static str> {
        self.store.remove(key).map(|_| ()).ok_or("Key not found")
    }

    pub fn persist(&mut self, record: &Record) -> Result<(), &'static str> {
        if self.next_offset + RECORD_SIZE > DB_SIZE {
            return Err("Flash full");
        }

        let flash_addr = DB_START + self.next_offset;
        FlashWriter::write(flash_addr, &record.to_bytes())?;
        self.next_offset += RECORD_SIZE;

        Ok(())
    }

    pub fn restore(&mut self) {
        self.store.clear();
        self.next_offset = 0;

        let mut buffer = [0xFFu8; RECORD_SIZE];

        while self.next_offset + RECORD_SIZE <= DB_SIZE {
            let addr = DB_START + self.next_offset;
            FlashWriter::read(addr, &mut buffer);

            if let Some(record) = Record::from_bytes(&buffer) {
                let _ = self.store.insert(record.key.clone(), record.value.clone());
                self.next_offset += RECORD_SIZE;
            } else {
                break;
            }
        }
    }
}
