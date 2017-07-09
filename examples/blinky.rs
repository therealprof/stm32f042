#![feature(used)]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rt;
extern crate stm32f042;

use stm32f042::stm32f042x::{GPIOB, RCC};
use cortex_m::asm;

fn main() {
    cortex_m::interrupt::free(|cs| {
        let rcc = RCC.borrow(cs);
        let gpiob = GPIOB.borrow(cs);

        // Enable clock for GPIO Port B
        rcc.ahbenr.modify(|_, w| w.iopben().set());

        // (Re-)configure PB1 as output
        gpiob.moder.modify(|_, w| unsafe { w.moder1().bits(1) });

        loop {
            // Turn PB1 on a million times in a row
            for _ in 0..1000000 {
                gpiob.bsrr.write(|w| w.bs1().set());
            }
            // Then turn PB1 off a million times in a row
            for _ in 0..1000000 {
                gpiob.brr.write(|w| w.br1().set());
            }
        }
    });
}

#[allow(dead_code)]
#[used]
#[link_section = ".rodata.interrupts"]
static INTERRUPTS: [extern "C" fn(); 240] = [default_handler; 240];

extern "C" fn default_handler() {
    asm::bkpt();
}
