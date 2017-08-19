#![feature(used)]
#![feature(const_fn)]
#![no_std]

extern crate cortex_m;

#[macro_use(exception)]
extern crate cortex_m_rt;
extern crate stm32f042;
extern crate volatile_register;

use stm32f042::*;

use self::{RCC, SYST};
use cortex_m::peripheral::SystClkSource;
use core::fmt::Write;


fn main() {
    cortex_m::interrupt::free(|cs| {
        let rcc = RCC.borrow(cs);
        let gpioa = GPIOA.borrow(cs);
        let syst = SYST.borrow(cs);
        let usart1 = USART1.borrow(cs);

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

        /* Initialise SysTick counter with a defined value */
        unsafe { syst.cvr.write(1) };

        /* Set source for SysTick counter, here 1/8th operating frequency (== 6 MHz) */
        syst.set_clock_source(SystClkSource::External);

        /* Set reload value, i.e. timer delay (== 1/100s) */
        syst.set_reload(10_000 - 1);

        /* Output a nice message */
        let _ = Write::write_str(&mut Buffer { cs }, "\nPlease state your business\n");

        /* Start counter */
        syst.enable_counter();

        /* Start interrupt generation */
        syst.enable_interrupt();
    });
}


/* Define an exception, i.e. function to call when exception occurs. Here if our SysTick timer
 * trips the echo function will be called */
exception!(SYS_TICK, echo);


struct Buffer<'a> {
    cs: &'a cortex_m::interrupt::CriticalSection,
}


impl<'a> core::fmt::Write for Buffer<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let usart1 = USART1.borrow(self.cs);
        for c in s.as_bytes() {
            /* Wait until the USART is clear to send */
            while usart1.isr.read().txe().bit_is_clear() {}

            /* Write the current character to the output register */
            usart1.tdr.modify(|_, w| unsafe { w.bits(*c as u32) });
        }
        Ok(())
    }
}


fn echo() {
    cortex_m::interrupt::free(|cs| {
        let usart1 = USART1.borrow(cs);

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
    });
}
