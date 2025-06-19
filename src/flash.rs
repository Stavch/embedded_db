use cortex_m::interrupt;
use stm32l4::stm32l4x6::FLASH;
use core::convert::TryInto;  // <-- Import at top

// Constants for flash addressing
pub const DB_START: usize = 0x080F0000;
pub const DB_END: usize = 0x080F1000;
pub const DB_SIZE: usize = DB_END - DB_START;

const FLASH_BASE: usize = 0x08000000;
const PAGE_SIZE: usize = 2048;

pub struct FlashWriter;

impl FlashWriter {
    /// Erase the flash page that contains the given address
    pub fn erase_page(address: usize) -> Result<(), &'static str> {
        if address < FLASH_BASE {
            return Err("Address below flash base");
        }

        let page_number = ((address - FLASH_BASE) / PAGE_SIZE) as u8;

        interrupt::free(|_| {
            let flash = unsafe { &*FLASH::ptr() };

            // Unlock flash
            if flash.cr.read().lock().bit_is_set() {
                flash.keyr.write(|w| unsafe { w.bits(0x4567_0123) });
                flash.keyr.write(|w| unsafe { w.bits(0xCDEF_89AB) });
            }

            while flash.sr.read().bsy().bit_is_set() {}

            // Set PER and PNB
            flash.cr.modify(|_, w| unsafe { w.per().set_bit().pnb().bits(page_number) });

            // Start erase
            flash.cr.modify(|_, w| w.start().set_bit());

            // Wait for erase to finish
            while flash.sr.read().bsy().bit_is_set() {}

            // Clear PER
            flash.cr.modify(|_, w| w.per().clear_bit());

            // Lock flash again
            flash.cr.modify(|_, w| w.lock().set_bit());
        });

        Ok(())
    }

    /// Read raw bytes from flash
    pub fn read(address: usize, buf: &mut [u8]) {
        for (i, byte) in buf.iter_mut().enumerate() {
            let ptr = (address + i) as *const u8;
            *byte = unsafe { core::ptr::read_volatile(ptr) };
        }
    }

    /// Write raw bytes to flash (must be 8-byte aligned)
    pub fn write(address: usize, data: &[u8]) -> Result<(), &'static str> {
        // Ensure address alignment to 8 bytes (double word)
        if address % 8 != 0 || data.len() % 8 != 0 {
            return Err("Address and data length must be 8-byte aligned");
        }

        interrupt::free(|_| {
            let flash = unsafe { &*FLASH::ptr() };

            // Unlock flash if locked
            if flash.cr.read().lock().bit_is_set() {
                flash.keyr.write(|w| unsafe { w.bits(0x4567_0123) });
                flash.keyr.write(|w| unsafe { w.bits(0xCDEF_89AB) });
            }

            while flash.sr.read().bsy().bit_is_set() {}

            // Enable programming mode
            flash.cr.modify(|_, w| w.pg().set_bit());

            for (i, chunk) in data.chunks_exact(8).enumerate() {
                let double_word = u64::from_le_bytes(chunk.try_into().unwrap());
                let ptr = (address + i * 8) as *mut u64;
                unsafe { core::ptr::write_volatile(ptr, double_word) };
                while flash.sr.read().bsy().bit_is_set() {}
            }

            // Disable programming mode
            flash.cr.modify(|_, w| w.pg().clear_bit());

            // Lock flash
            flash.cr.modify(|_, w| w.lock().set_bit());
        });

        Ok(())
    }
}
