#![feature(used)]
#![no_std]

extern crate cortex_m;
extern crate stm32f042;

fn main() {
    if let Some(p) = stm32f042::Peripherals::take() {
        let rcc = p.RCC;
        let gpioa = p.GPIOA;

        /* Enable clock for SYSCFG, else everything will behave funky! */
        rcc.apb2enr.modify(|_, w| w.syscfgen().set_bit());

        /* Enable clock for GPIO Port A */
        rcc.ahbenr.modify(|_, w| w.iopaen().set_bit());

        /* (Re-)configure PA1 as output */
        gpioa.moder.modify(|_, w| unsafe { w.moder1().bits(1) });

        loop {
            /* Turn PA1 on a million times in a row */
            for _ in 0..1_000_000 {
                gpioa.bsrr.write(|w| w.bs1().set_bit());
            }
            /* Then turn PA1 off a million times in a row */
            for _ in 0..1_000_000 {
                gpioa.bsrr.write(|w| w.br1().set_bit());
            }
        }
    }
}
