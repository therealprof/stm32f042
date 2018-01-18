#![feature(used)]
#![feature(const_fn)]
#![no_std]

extern crate cortex_m;
use cortex_m::peripheral::Peripherals;
use stm32f042::peripherals::usart;

#[macro_use(exception)]
extern crate stm32f042;

use stm32f042::*;

use self::RCC;
use cortex_m::peripheral::syst::SystClkSource;
use core::fmt::Write;


fn main() {
    if let Some(mut peripherals) = Peripherals::take() {
        cortex_m::interrupt::free(|cs| {
            let rcc = unsafe { &(*RCC::ptr()) };
            let gpioa = unsafe { &(*GPIOA::ptr()) };
            let usart1 = unsafe { &(*USART1::ptr()) };

            /* Enable clock for SYSCFG and USART */
            rcc.apb2enr.modify(|_, w| {
                w.syscfgen().set_bit().usart1en().set_bit()
            });

            /* Enable clock for GPIO Port A */
            rcc.ahbenr.modify(|_, w| w.iopaen().set_bit());

            /* Set alternate function 1 to to enable USART RX/TX */
            gpioa.moder.modify(|_, w| unsafe {
                w.moder9().bits(2).moder10().bits(2)
            });

            /* Set AF1 for pin 9/10 to enable USART RX/TX */
            gpioa.afrh.modify(|_, w| unsafe {
                w.afrh9().bits(1).afrh10().bits(1)
            });

            /* Set baudrate to 115200 @8MHz */
            usart1.brr.write(|w| unsafe { w.bits(0x045) });

            /* Reset other registers to disable advanced USART features */
            usart1.cr2.reset();
            usart1.cr3.reset();

            /* Enable transmission and receiving */
            usart1.cr1.modify(|_, w| unsafe { w.bits(0xD) });

            /* Set source for SysTick counter, here 1/8th operating frequency (== 6 MHz) */
            peripherals.SYST.set_clock_source(SystClkSource::External);

            /* Set reload value, i.e. timer delay (== 1/100s) */
            peripherals.SYST.set_reload(10_000 - 1);

            /* Start counter */
            peripherals.SYST.enable_counter();

            /* Start interrupt generation */
            peripherals.SYST.enable_interrupt();

            /* Output a nice message */
            let _ = Write::write_str(
                &mut usart::USARTBuffer(cs),
                "\nPlease state your business\n",
            );
        });
    }
}



/* Define an exception, i.e. function to call when exception occurs. Here if our SysTick timer
 * trips the echo function will be called */
exception!(SYS_TICK, echo);

fn echo() {
    let usart1 = unsafe { &(*USART1::ptr()) };

    let c = {
        /* Check for overflow */
        if usart1.isr.read().ore().bit_is_set() {
            usart1.icr.modify(|_, w| w.orecf().set_bit());
            usart1.rdr.read().bits()
        }
        /* Check if the USART received something */
        else if usart1.isr.read().rxne().bit_is_set() {
            usart1.rdr.read().bits()
        }
        /* Otherwise we'll set a dummy value */
        else {
            0
        }
    };

    /* If value is not the dummy value: echo it back to the serial line */
    if c != 0 {
        /* Wait until the USART is clear to send */
        while usart1.isr.read().txe().bit_is_clear() {}

        /* Write the current character to the output register */
        usart1.tdr.modify(|_, w| unsafe { w.bits(c) });
    }
}
