#![feature(used)]
#![feature(const_fn)]
#![no_std]

extern crate cortex_m;
use cortex_m::peripheral::Peripherals;

#[macro_use(exception)]
extern crate stm32f042;

use stm32f042::*;

use self::{GPIOB, RCC};
use cortex_m::peripheral::syst::SystClkSource;


fn main() {
    if let Some(mut peripherals) = Peripherals::take() {
        let rcc = unsafe { &(*RCC::ptr()) };
        let gpiob = unsafe { &(*GPIOB::ptr()) };

        /* Enable clock for SYSCFG, else everything will behave funky! */
        rcc.apb2enr.modify(|_, w| w.syscfgen().set_bit());

        /* Enable clock for GPIO Port B */
        rcc.ahbenr.modify(|_, w| w.iopben().set_bit());

        /* (Re-)configure PB1 as output */
        gpiob.moder.modify(|_, w| unsafe { w.moder1().bits(1) });

        /* Set source for SysTick counter, here 1/8th operating frequency (== 6 MHz) */
        peripherals.SYST.set_clock_source(SystClkSource::External);

        /* Set reload value, i.e. timer delay (== 1s) */
        peripherals.SYST.set_reload(1_000_000 - 1);

        /* Start counter */
        peripherals.SYST.enable_counter();

        /* Start interrupt generation */
        peripherals.SYST.enable_interrupt();
    }
}


/* Define an exception, i.e. function to call when exception occurs. Here if our SysTick timer
 * trips the blinky function will be called and the specified stated passed in via argument */
exception!(SYS_TICK, blink, locals: {
    state: bool = false;
});


fn blink(l: &mut SYS_TICK::Locals) {
    /* Enter critical section */
    cortex_m::interrupt::free(|_| {
        let gpiob = unsafe { &(*GPIOB::ptr()) };

        /* Check state variable */
        if l.state {
            /* If set turn off the LED */
            gpiob.brr.write(|w| w.br1().set_bit());

            /* And set new state to false */
            l.state = false;
        } else {
            /* If not set, turn on the LED */
            gpiob.bsrr.write(|w| w.bs1().set_bit());

            /* And set new state to true */
            l.state = true;
        }
    });
}
