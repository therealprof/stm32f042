#![feature(used)]
#![feature(const_fn)]
#![no_std]

use core::cell::RefCell;

extern crate cortex_m;

use cortex_m::interrupt::Mutex;
use cortex_m::peripheral::syst::*;

#[macro_use(exception)]
extern crate stm32f042;

static GPIOA: Mutex<RefCell<Option<stm32f042::GPIOA>>> = Mutex::new(RefCell::new(None));

fn main() {
    if let (Some(cp), Some(p)) = (
        cortex_m::Peripherals::take(),
        stm32f042::Peripherals::take(),
    ) {
        let rcc = p.RCC;
        let gpioa = p.GPIOA;
        let mut syst = cp.SYST;
        let flash = p.FLASH;

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

        /* Enable clock for GPIO Port A */
        rcc.ahbenr.modify(|_, w| w.iopaen().set_bit());

        /*  (Re-)configure PA1 as output */
        gpioa.moder.modify(|_, w| unsafe { w.moder1().bits(1) });

        cortex_m::interrupt::free(|cs| {
            *GPIOA.borrow(cs).borrow_mut() = Some(gpioa);
        });

        /* Initialise SysTick counter with a defined value */
        unsafe { syst.cvr.write(1) };

        /* Set source for SysTick counter, here full operating frequency (== 8MHz) */
        syst.set_clock_source(SystClkSource::Core);

        /* Set reload value, i.e. timer delay 48 MHz/4 Mcounts == 12Hz or 83ms */
        syst.set_reload(4_000_000 - 1);

        /* Start counter */
        syst.enable_counter();

        /* Start interrupt generation */
        syst.enable_interrupt();
    }
}

/* Define an exception, i.e. function to call when exception occurs. Here if our SysTick timer
 * trips the flash function will be called and the specified stated passed in via argument */
exception!(SYS_TICK, flash, locals: {
    state: u8 = 1;
});

fn flash(l: &mut SYS_TICK::Locals) {
    /* Enter critical section */
    cortex_m::interrupt::free(|cs| {
        if let Some(gpioa) = GPIOA.borrow(cs).borrow().as_ref() {
            /* Check state variable, keep LED off most of the time and turn it on every 10th tick */
            if l.state < 10 {
                /* If set turn off the LED */
                gpioa.brr.write(|w| w.br1().set_bit());

                /* And now increment state variable */
                l.state += 1;
            } else {
                /* If not set, turn on the LED */
                gpioa.bsrr.write(|w| w.bs1().set_bit());

                /* And set new state variable back to 0 */
                l.state = 1;
            }
        }
    });
}
