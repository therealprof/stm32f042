#![feature(used)]
#![feature(const_fn)]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rt;

#[macro_use(interrupt)]
extern crate stm32f042;
extern crate volatile_register;

use stm32f042::*;

use core::fmt::Write;

use stm32f042::Interrupt;
use stm32f042::peripherals::usart;


fn main() {
    cortex_m::interrupt::free(|cs| {
        let rcc = RCC.borrow(cs);
        let gpioa = GPIOA.borrow(cs);
        let gpiof = GPIOF.borrow(cs);
        let usart1 = stm32f042::USART1.borrow(cs);
        let nvic = NVIC.borrow(cs);
        let i2c = I2C1.borrow(cs);

        /* Enable clock for SYSCFG and USART */
        rcc.apb2enr.modify(|_, w| {
            w.syscfgen().set_bit().usart1en().set_bit()
        });

        /* Enable clock for GPIO Port A, B and F */
        rcc.ahbenr.modify(|_, w| {
            w.iopaen().set_bit().iopben().set_bit().iopfen().set_bit()
        });

        /* Enable clock for TIM2 and I2C1 */
        rcc.apb1enr.modify(
            |_, w| w.tim2en().set_bit().i2c1en().set_bit(),
        );

        /* Set alternate function on PF0 and PF1 */
        gpiof.moder.modify(|_, w| unsafe {
            w.moder0().bits(2).moder1().bits(2)
        });

        /* Set AF1 for pin PF0/PF1 to enable I2C */
        gpiof.afrl.modify(|_, w| unsafe {
            w.afrl0().bits(1).afrl1().bits(1)
        });

        /* Set internal pull-up for pin PF0/PF1 */
        gpiof.pupdr.modify(|_, w| unsafe {
            w.pupdr0().bits(1).pupdr1().bits(1)
        });

        /* Set mode to open drain for pin PF0/PF1 */
        gpiof.otyper.modify(
            |_, w| w.ot0().set_bit().ot1().set_bit(),
        );

        /* Set PF0, PF1 to high speed */
        gpiof.ospeedr.modify(|_, w| unsafe {
            w.ospeedr0().bits(3).ospeedr1().bits(3)
        });

        /* Make sure the I2C unit is disabled so we can configure it */
        i2c.cr1.modify(|_, w| w.pe().clear_bit());

        /* Enable I2C signal generator, and configure I2C for 400KHz full speed */
        i2c.timingr.write(|w| unsafe { w.bits(0x0010_0209) });

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
            "\r\nWelcome to the I2C scanner. Enter any character to start scan.\r\n",
        );
    });
}


/* Define an interrupt handler, i.e. function to call when interrupt occurs. Here if we receive a
 * character from the USART well call the handler */
interrupt!(USART1, usart_receive);


/* The IRQ handler triggered by a received character in USART buffer, this will conduct our I2C
 * scan when we receive anything */
fn usart_receive() {
    cortex_m::interrupt::free(|cs| {
        let usart1 = stm32f042::USART1.borrow(cs);
        let i2c = I2C1.borrow(cs);

        /* Read the character that triggered the interrupt from the USART */
        usart::read_char(usart1, false);

        /* Output address schema for tried addresses */
        let _ = Write::write_str(&mut usart::USARTBuffer(cs), "\r\n");
        let _ = Write::write_str(
            &mut usart::USARTBuffer(cs),
            "0       1               2               3               4               5               6               7\r\n",
        );
        let _ = Write::write_str(
            &mut usart::USARTBuffer(cs),
            "89ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF\r\n",
        );

        /* Enable the I2C processing */
        i2c.cr1.modify(|_, w| w.pe().set_bit());

        /* Execute scanning once for each valid I2C address */
        for addr in 8..0x80 {
            /* Wait while busy, just to be on the sure side */
            while i2c.isr.read().busy().bit_is_set() {}

            /* Wait while someone else is using the I2C bus, just to be on the sure side */
            while i2c.cr2.read().start().bit_is_set() {}

            /* Set up current address, we're trying a "write" command and not going to set anything
             * and make sure we end a non-NACKed read (i.e. if we found a device) properly */
            i2c.cr2.modify(|_, w| unsafe {
                w.sadd1()
                    .bits(addr)
                    .nbytes()
                    .bits(0)
                    .rd_wrn()
                    .clear_bit()
                    .autoend()
                    .set_bit()
            });

            /* Send a START condition */
            i2c.cr2.modify(|_, w| w.start().set_bit());

            /* Wait until the transmit buffer is empty and there hasn't been either a NACK or STOP
             * being received */
            while i2c.isr.read().txis().bit_is_clear() {
                if i2c.isr.read().nackf().bit_is_set() || i2c.isr.read().stopf().bit_is_set() {
                    break;
                }
            }

            /* If we received a NACK there's no device on the tried address */
            let _ = Write::write_str(
                &mut usart::USARTBuffer(cs),
                if i2c.isr.read().nackf().bit_is_set() {
                    "N"
                } else {
                    "Y"
                },
            );

            /* Clear STOP and NACK status flags */
            i2c.icr.write(|w| w.nackcf().set_bit().stopcf().set_bit());
        }

        /* Disable the I2C port. */
        i2c.cr1.modify(|_, w| w.pe().clear_bit());

        let _ = Write::write_str(
            &mut usart::USARTBuffer(cs),
            "\r\n\r\nScan done.\r\n'Y' means a device was found on the I2C address above.\r\n'N' means no device found on that address.\r\nPlease enter any character to start a new scan.\r\n",
        );
    });
}
