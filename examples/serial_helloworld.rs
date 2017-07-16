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
        let flash = FLASH.borrow(cs);
        let usart1 = USART1.borrow(cs);

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

        /* Set alternate function 1 to to enable USART RX/TX */
        gpioa.moder.modify(|_, w| unsafe { w.moder9().bits(2) });
        gpioa.moder.modify(|_, w| unsafe { w.moder10().bits(2) });

        /* Set AF1 for pin 9/10 to enable USART RX/TX */
        gpioa.afrh.modify(|_, w| unsafe { w.afrh9().bits(1) });
        gpioa.afrh.modify(|_, w| unsafe { w.afrh10().bits(1) });

        /* Enable USART clock */
        rcc.apb2enr.modify(|_, w| w.usart1en().set_bit());

        /* Set baudrate to 115200 @48MHz */
        usart1.brr.write(|w| unsafe { w.bits(0x01A0) });

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
        syst.set_reload(6_000_000);

        /* Start counter */
        syst.enable_counter();

        /* Start interrupt generation */
        syst.enable_interrupt();
    });
}


/* Define an exception, i.e. function to call when exception occurs. Here if our SysTick timer
 * trips the hello_world function will be called */
exception!(SYS_TICK, hello_world);


fn hello_world() {
    /* The string we want to print over serial */
    let s = &"Hello World!\n";

    /* Enter critical section */
    cortex_m::interrupt::free(|cs| {
        let usart1 = USART1.borrow(cs);

        /* Iterate over all characters in the string to send */
        for c in s.chars() {
            /* Wait until the USART is clear to send */
            while usart1.isr.read().txe().bit_is_clear() {}

            /* Write the current character to the output register */
            usart1.tdr.modify(|_, w| unsafe { w.bits(c as u32) });
        }
    });
}
