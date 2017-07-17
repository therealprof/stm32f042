#![feature(used)]
#![feature(const_fn)]
#![no_std]

extern crate cortex_m;

#[macro_use(exception)]
extern crate cortex_m_rt;
extern crate stm32f042;
extern crate volatile_register;

use stm32f042::*;

use self::{GPIOB, RCC, SYST};
use cortex_m::peripheral::SystClkSource;


fn main() {
    cortex_m::interrupt::free(|cs| {
        let rcc = RCC.borrow(cs);
        let gpiob = GPIOB.borrow(cs);
        let syst = SYST.borrow(cs);

        /* Enable clock for SYSCFG, else everything will behave funky! */
        rcc.apb2enr.modify(|_, w| w.syscfgen().set_bit());

        /* Enable clock for GPIO Port B */
        rcc.ahbenr.modify(|_, w| w.iopben().set_bit());

        /* (Re-)configure PB1 as output */
        gpiob.moder.modify(|_, w| unsafe { w.moder1().bits(1) });

        /* Initialise SysTick counter with a defined value */
        unsafe { syst.cvr.write(1) };

        /* Set source for SysTick counter, here full operating frequency (== 8MHz) */
        syst.set_clock_source(SystClkSource::Core);

        /* Set reload value, i.e. timer delay (== 128ms) */
        syst.set_reload(1_000_000);

        /* Start counter */
        syst.enable_counter();

        /* Start interrupt generation */
        syst.enable_interrupt();
    });
}


/* Define an exception, i.e. function to call when exception occurs. Here if our SysTick timer
 * trips the blinky function will be called and the specified stated passed in via argument */
exception!(SYS_TICK, blink, locals: {
    state: bool = false;
});


fn blink(l: &mut SYS_TICK::Locals) {
    /* Enter critical section */
    cortex_m::interrupt::free(|cs| {
        let gpiob = GPIOB.borrow(cs);

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
