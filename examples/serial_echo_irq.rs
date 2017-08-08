#![feature(used)]
#![feature(const_fn)]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rt;

#[macro_use(interrupt)]
extern crate stm32f042;
extern crate volatile_register;

use stm32f042::*;

use self::RCC;
use core::fmt::Write;
use stm32f042::Interrupt;


fn main() {
    cortex_m::interrupt::free(|cs| {
        let rcc = RCC.borrow(cs);
        let gpioa = GPIOA.borrow(cs);
        let gpiob = GPIOB.borrow(cs);
        let usart1 = stm32f042::USART1.borrow(cs);
        let nvic = NVIC.borrow(cs);

        /* Enable clock for SYSCFG and USART */
        rcc.apb2enr.modify(|_, w| {
            w.syscfgen().set_bit().usart1en().set_bit()
        });

        /* Enable clock for GPIO Port A and B */
        rcc.ahbenr.modify(
            |_, w| w.iopaen().set_bit().iopben().set_bit(),
        );

        /* (Re-)configure PB1 as output */
        gpiob.moder.modify(|_, w| unsafe { w.moder1().bits(1) });

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

        /* Enable transmission and receiving as well as the RX IRQ */
        usart1.cr1.modify(|_, w| unsafe { w.bits(0x2D) });

        /* Enable USART IRQ, set prio 0 and clear any pending IRQs */
        nvic.enable(Interrupt::USART1);
        unsafe { nvic.set_priority(Interrupt::USART1, 1) };
        nvic.clear_pending(Interrupt::USART1);

        /* Output a nice message */
        Write::write_str(&mut Buffer { cs }, "\nPlease state your business\n").unwrap();
    });
}


/* Define an interrupt handler, i.e. function to call when interrupt occurs. Here if we receive a
 * character from the USART, our echo_b_blink function will be called */
interrupt!(USART1, echo_n_blink);


struct Buffer<'a> {
    cs: &'a cortex_m::interrupt::CriticalSection,
}


impl<'a> core::fmt::Write for Buffer<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let usart1 = stm32f042::USART1.borrow(self.cs);
        for c in s.as_bytes() {
            /* Wait until the USART is clear to send */
            while usart1.isr.read().txe().bit_is_clear() {}

            /* Write the current character to the output register */
            usart1.tdr.modify(|_, w| unsafe { w.bits(*c as u32) });
        }
        Ok(())
    }
}


fn echo_n_blink() {
    cortex_m::interrupt::free(|cs| {
        let gpiob = GPIOB.borrow(cs);
        let usart1 = stm32f042::USART1.borrow(cs);

        /* Read the received value from the USART register */
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

        /* Turn PB1 on for a bit */
        for _ in 0..50_000 {
            gpiob.bsrr.write(|w| w.bs1().set_bit());
        }

        /* And then off again */
        gpiob.brr.write(|w| w.br1().set_bit());
    });
}
