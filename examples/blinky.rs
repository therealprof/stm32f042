#![feature(used)]
#![no_std]

extern crate cortex_m;
extern crate stm32f042;

use stm32f042::{GPIOB, RCC};

fn main() {
    let rcc = unsafe { &(*RCC::ptr()) };
    let gpiob = unsafe { &(*GPIOB::ptr()) };

    /* Enable clock for SYSCFG, else everything will behave funky! */
    rcc.apb2enr.modify(|_, w| w.syscfgen().set_bit());

    /* Enable clock for GPIO Port B */
    rcc.ahbenr.modify(|_, w| w.iopben().set_bit());

    /* (Re-)configure PB1 as output */
    gpiob.moder.modify(|_, w| unsafe { w.moder1().bits(1) });

    loop {
        /* Turn PB1 on a million times in a row */
        for _ in 0..1_000_000 {
            gpiob.bsrr.write(|w| w.bs1().set_bit());
        }
        /* Then turn PB1 off a million times in a row */
        for _ in 0..1_000_000 {
            gpiob.brr.write(|w| w.br1().set_bit());
        }
    }
}
