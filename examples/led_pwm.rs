#![feature(used)]
#![feature(const_fn)]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rt;

#[macro_use(interrupt)]
extern crate stm32f042;
extern crate volatile_register;

use stm32f042::*;
use stm32f042::peripherals::usart;

use core::fmt::Write;
use stm32f042::Interrupt;


fn main() {
    cortex_m::interrupt::free(|cs| {
        let rcc = RCC.borrow(cs);
        let gpioa = GPIOA.borrow(cs);
        let usart1 = stm32f042::USART1.borrow(cs);
        let nvic = NVIC.borrow(cs);
        let tim2 = TIM2.borrow(cs);

        /* Enable clock for SYSCFG and USART */
        rcc.apb2enr.modify(|_, w| {
            w.syscfgen().set_bit().usart1en().set_bit()
        });

        /* Enable clock for GPIO Port A and B */
        rcc.ahbenr.modify(
            |_, w| w.iopaen().set_bit().iopben().set_bit(),
        );

        /* Enable clock for TIM2 */
        rcc.apb1enr.modify(|_, w| w.tim2en().set_bit());

        /* Set PA1, PA2, PA3 to TIM2 controlled */
        gpioa.afrl.modify(|_, w| unsafe {
            w.afrl1().bits(2).afrl2().bits(2).afrl3().bits(2)
        });

        /* Set PA1, PA2, PA3 to alternate function */
        gpioa.moder.modify(|_, w| unsafe {
            w.moder1().bits(2).moder2().bits(2).moder3().bits(2)
        });

        /* Set PA1, PA2, PA3 to high speed */
        gpioa.ospeedr.modify(|_, w| unsafe {
            w.ospeedr1().bits(3).ospeedr2().bits(3).ospeedr3().bits(3)
        });

        /* Set timer clock to 1 MHz (== Sysclock / (7 + 1)) */
        tim2.psc.write(|w| unsafe { w.psc().bits(7) });

        /* Set timer clock to 1 MHz (== Sysclock / (7 + 1)) */
        tim2.arr.write(|w| unsafe { w.arr_l().bits(255) });

        /* Activate TIM2 Channels 2,3,4 in PWM mode */
        tim2.ccmr1_output.modify(|_, w| unsafe {
            w.oc2m().bits(6).oc2pe().bit(true)
        });
        tim2.ccmr2_output.modify(|_, w| unsafe {
            w.oc3m()
                .bits(6)
                .oc3pe()
                .bit(true)
                .oc4m()
                .bits(6)
                .oc4pe()
                .bit(true)
        });

        /* Enable TIM2 signal generator */
        tim2.cr1.modify(|_, w| w.cen().bit(true));

        /* Turn on compare and update generators */
        tim2.egr.write(|w| {
            w.cc2g()
                .bit(true)
                .cc3g()
                .bit(true)
                .cc4g()
                .bit(true)
                .ug()
                .bit(true)
        });

        /* Turn on channel outputs */
        tim2.ccer.modify(|_, w| {
            w.cc2e().bit(true).cc3e().bit(true).cc4e().bit(true)
        });

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
        let _ = Write::write_str(
            &mut usart::USARTBuffer(cs),
            "\r\nLED PWM demo: connect RGB LED to PA1, PA2, PA3.\r\nEnter RGB value in hex as RRGGBB\r\nEnter 'o' for individual channel dimming demo\r\nEnter 'p' for all channel PWM dimming demo\r\nAny other key will reset values\r\n",
        );
    });
}


/* Define an interrupt handler, i.e. function to call when interrupt occurs. Here if we receive a
 * character from the USART well call the handler to convert the values to PWM changes */
interrupt!(USART1, echo_n_pwm, locals: {
    state : u8 = 0;
    r : u8 = 0;
    g : u8 = 0;
    b : u8 = 0;
});


/* Convert read character hex values to numbers */
fn hex_to_number(input: u8) -> u8 {
    match input as char {
        '0'...'9' => input - 48,
        'a'...'f' => input - 87,
        'A'...'F' => input - 55,
        _ => 0,
    }
}


/* Set PWM channel compare register to the specified intensities */
fn change_color(tim2: &stm32f042::TIM2, r: u8, g: u8, b: u8) {
    tim2.ccr2.modify(|_, w| unsafe { w.bits(r as u32) });
    tim2.ccr3.modify(|_, w| unsafe { w.bits(g as u32) });
    tim2.ccr4.modify(|_, w| unsafe { w.bits(b as u32) });
}


/* The IRQ handler to read the character from the USART buffer, convert it to some usable
 * instruction and reprogram the PWM accordingly */
fn echo_n_pwm(l: &mut USART1::Locals) {
    cortex_m::interrupt::free(|cs| {
        let usart1 = stm32f042::USART1.borrow(cs);
        let tim2 = stm32f042::TIM2.borrow(cs);

        /* Read a character */
        let c = usart::read_char(usart1, true);

        /* Match character */
        match c as char {
            /* ... is it a hex number? */
            '0'...'9' | 'a'...'f' | 'A'...'F' => {
                let c = hex_to_number(c);

                l.state += 1;
                /* ... then use it to set the current color component in RRGGBB format */
                match l.state {
                    1 => l.r = c << 4,
                    2 => l.r = (l.r & 0xf0) | c,
                    3 => l.g = c << 4,
                    4 => l.g = (l.g & 0xf0) | c,
                    5 => l.b = c << 4,
                    6 => {
                        l.b = (l.b & 0xf0) | c;
                        l.state = 0
                    }
                    _ => l.state = 0,
                }
            }
            /* If it's a 'p' then PWM dim RGB from off to full on and back */
            'p' => {
                for i in 0..255 {
                    for _ in 0..2048 {
                        change_color(tim2, i, i, i);
                    }
                }

                for i in (0..255).rev() {
                    for _ in 0..2048 {
                        change_color(tim2, i, i, i);
                    }
                }

                l.r = 0;
                l.g = 0;
                l.b = 0;
                l.state = 0;
            }

            /* If it's an 'o' then PWM dim each channel (RGB) from off to full on and back */
            'o' => {
                for color in 0..3 {
                    let mut r = 0;
                    let mut g = 0;
                    let mut b = 0;

                    for i in 0..255 {
                        for _ in 0..2048 {
                            if color == 0 {
                                r = i;
                            } else if color == 1 {
                                g = i;
                            } else {
                                b = i;
                            }

                            change_color(tim2, r, g, b);
                        }
                    }

                    for i in (0..255).rev() {
                        for _ in 0..2048 {
                            if color == 0 {
                                r = i;
                            } else if color == 1 {
                                g = i;
                            } else {
                                b = i;
                            }

                            change_color(tim2, r, g, b);
                        }
                    }
                }

                l.r = 0;
                l.g = 0;
                l.b = 0;
                l.state = 0;
            }
            _ => {
                l.r = 0;
                l.g = 0;
                l.b = 0;
                l.state = 0;
            }
        }

        /* Set the current color state into the LEDs */
        change_color(tim2, l.r, l.g, l.b);
    });
}
