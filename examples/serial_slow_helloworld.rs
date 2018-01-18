#![feature(used)]
#![feature(const_fn)]
#![no_std]
#![feature(trace_macros)]

extern crate cortex_m;
use cortex_m::peripheral::Peripherals;

#[macro_use(exception)]
extern crate stm32f042;

use core::fmt::Write;

use stm32f042::*;
use stm32f042::peripherals::usart::*;

use self::RCC;
use cortex_m::peripheral::syst::SystClkSource;


fn main() {
    if let Some(mut peripherals) = Peripherals::take() {
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

        /* Set reload value, i.e. timer delay (== 1s) */
        peripherals.SYST.set_reload(1_000_000 - 1);

        /* Start counter */
        peripherals.SYST.enable_counter();

        /* Start interrupt generation */
        peripherals.SYST.enable_interrupt();
    }
}


/* Define an exception, i.e. function to call when exception occurs. Here if our SysTick timer
 * trips the hello_world function will be called */
exception!(SYS_TICK, hello_world, locals: {
    count: u32 = 0;
});



fn hello_world(l: &mut SYS_TICK::Locals) {
    l.count += 1;

    /* Enter critical section */
    cortex_m::interrupt::free(|cs| {
        /* Please be aware that while comfortable, this is a really heavyweight operation! */
        let _ = writeln!(USARTBuffer(cs), "Hello World! The count is: {:#x}", l.count);
    });
}
