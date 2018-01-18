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
        let flash = unsafe { &(*FLASH::ptr()) };

        /* Enable clock for SYSCFG, else everything will behave funky! */
        rcc.apb2enr.modify(|_, w| w.syscfgen().set_bit());

        /* Fire up HSI48 clock */
        rcc.cr2.write(|w| w.hsi48on().set_bit());

        /* Wait for HSI48 clock to become ready */
        loop {
            if rcc.cr2.read().hsi48rdy().bit() {
                break;
            }
        }

        /* Enable flash prefetch */
        flash.acr.write(|w| w.prftbe().set_bit());

        /* Wait for flash prefetch to become enabled */
        loop {
            if flash.acr.read().prftbs().bit() {
                break;
            }
        }

        /* Set up flash waitstate for the higher frequency */
        flash.acr.write(|w| unsafe { w.latency().bits(1) });

        /* Make HSI48 clock the system clock */
        unsafe { rcc.cfgr.write(|w| w.sw().bits(3)) };

        /* Wait for HSI48 clock to system clock */
        loop {
            if rcc.cfgr.read().sw().bits() == 3 {
                break;
            }
        }

        /* Enable clock for GPIO Port B */
        rcc.ahbenr.modify(|_, w| w.iopben().set_bit());

        /*  (Re-)configure PB1 as output */
        gpiob.moder.modify(|_, w| unsafe { w.moder1().bits(1) });

        /* Set source for SysTick counter, here full operating frequency (== 8MHz) */
        peripherals.SYST.set_clock_source(SystClkSource::Core);

        /* Set reload value, i.e. timer delay 48 MHz/4 Mcounts == 12Hz or 83ms */
        peripherals.SYST.set_reload(4_000_000 - 1);

        /* Start counter */
        peripherals.SYST.enable_counter();

        /* Start interrupt generation */
        peripherals.SYST.enable_interrupt();
    }
}


/* Define an exception, i.e. function to call when exception occurs. Here if our SysTick timer
 * trips the flash function will be called and the specified stated passed in via argument */
exception!(SYS_TICK, flash, locals: {
    state: u8 = 1;
});


fn flash(l: &mut SYS_TICK::Locals) {
    let gpiob = unsafe { &(*GPIOB::ptr()) };

    /* Check state variable, keep LED off most of the time and turn it on every 10th tick */
    if l.state < 10 {
        /* If set turn off the LED */
        gpiob.brr.write(|w| w.br1().set_bit());

        /* And now increment state variable */
        l.state += 1;
    } else {
        /* If not set, turn on the LED */
        gpiob.bsrr.write(|w| w.bs1().set_bit());

        /* And set new state variable back to 0 */
        l.state = 1;
    }
}
