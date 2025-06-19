use stm32l4xx_hal::{
    pac,
    prelude::*,
    serial::{Config, Serial},
};
use nb::block;

pub struct Uart {
    tx: stm32l4xx_hal::serial::Tx<pac::USART2>,
    rx: stm32l4xx_hal::serial::Rx<pac::USART2>,
}

impl uart {
    pub fn new() -> Self {
        let dp = pac::Peripherals::take().unwrap();

        let mut rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.freeze();

        let gpioa = dp.GPIOA.split(&mut rcc.ahb2);
        let tx_pin = gpioa.pa2.into_af7();
        let rx_pin = gpioa.pa3.into_af7();

        let serial = Serial::usart2(
            dp.USART2,
            (tx_pin, rx_pin),
            Config::default().baudrate(115_200.bps()),
            clocks,
            &mut rcc.apb1r1,
        )
        .unwrap();

        let (tx, rx) = serial.split();

        Self { tx, rx }
    }

    pub fn write_str(&mut self, s: &str) {
        for byte in s.as_bytes() {
            block!(self.tx.write(*byte)).ok();
        }
    }

    pub fn read_byte(&mut self) -> Option<u8> {
        match self.rx.read() {
            Ok(b) => Some(b),
            Err(nb::Error::WouldBlock) => None,
            Err(_) => None,
        }
    }
}
