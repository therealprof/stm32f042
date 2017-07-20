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


fn main() {
    cortex_m::interrupt::free(|cs| {
        let rcc = RCC.borrow(cs);
        let gpioa = GPIOA.borrow(cs);
        let syst = SYST.borrow(cs);
        let usart1 = USART1.borrow(cs);

        /* Enable clock for SYSCFG, else everything will behave funky! */
        rcc.apb2enr.modify(|_, w| w.syscfgen().set_bit());

        /* Enable clock for GPIO Port A */
        rcc.ahbenr.modify(|_, w| w.iopaen().set_bit());

        /* Set alternate function 1 to to enable USART RX/TX */
        gpioa.moder.modify(|_, w| unsafe { w.moder9().bits(2) });
        gpioa.moder.modify(|_, w| unsafe { w.moder10().bits(2) });

        /* Set AF1 for pin 9/10 to enable USART RX/TX */
        gpioa.afrh.modify(|_, w| unsafe { w.afrh9().bits(1) });
        gpioa.afrh.modify(|_, w| unsafe { w.afrh10().bits(1) });

        /* Enable USART clock */
        rcc.apb2enr.modify(|_, w| w.usart1en().set_bit());

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

        /* Set reload value, i.e. timer delay (== 1s) */
        syst.set_reload(1_000_000 - 1);

        /* Start counter */
        syst.enable_counter();

        /* Start interrupt generation */
        syst.enable_interrupt();
    });
}


/* Define an exception, i.e. function to call when exception occurs. Here if our SysTick timer
 * trips the hello_world function will be called */
exception!(SYS_TICK, hello_world, locals: {
    count: u32 = 0;
});


/* Little helper function to print individual character to USART1 */
fn printc(cs: &cortex_m::interrupt::CriticalSection, c: char) {
    let usart1 = USART1.borrow(cs);

    /* Wait until the USART is clear to send */
    while usart1.isr.read().txe().bit_is_clear() {}

    /* Write the current character to the output register */
    usart1.tdr.modify(|_, w| unsafe { w.bits(c as u32) });
}


fn hello_world(l: &mut SYS_TICK::Locals) {
    /* The string we want to print over serial */
    let hello = &"Hello World! The count is: 0x";
    let end = &"\n\r";

    l.count += 1;

    /* Enter critical section */
    cortex_m::interrupt::free(|cs| {
        /* Iterate over all characters in the hello world string */
        for c in hello.chars() {
            printc(cs, c);
        }

        /* Format digits as hexadecimal number */
        let mut temp = l.count;
        for _ in 0..8 {
            /* Write the current character to the output register */
            let c = (temp >> 28) as u8;
            printc(
                cs,
                match c {
                    0...9 => c + 48,
                    10...15 => c + 55,
                    _ => 88,
                } as char,
            );
            temp <<= 4;
        }

        /* Print line end and carriage return */
        for c in end.chars() {
            printc(cs, c);
        }
    });
}
