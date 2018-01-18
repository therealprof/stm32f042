#![feature(used)]
#![feature(const_fn)]
#![no_std]

extern crate cortex_m;
use cortex_m::peripheral::Peripherals;

#[macro_use(interrupt)]
extern crate stm32f042;

use stm32f042::peripherals::i2c::write_data;
use stm32f042::peripherals::i2c::read_data;
use stm32f042::peripherals::usart;

use stm32f042::*;
use core::fmt::Write;
use stm32f042::Interrupt;


/* By default the MCP23017 on the Ixpando is configured to address 0x20 */
const I2C_ADDRESS: u8 = 0x20;


fn main() {
    if let Some(mut peripherals) = Peripherals::take() {
        cortex_m::interrupt::free(|cs| {
            let rcc = unsafe { &(*RCC::ptr()) };
            let gpioa = unsafe { &(*GPIOA::ptr()) };
            let gpiob = unsafe { &(*GPIOB::ptr()) };
            let gpiof = unsafe { &(*GPIOF::ptr()) };
            let usart1 = unsafe { &(*stm32f042::USART1::ptr()) };
            let i2c = unsafe { &(*I2C1::ptr()) };
            let syscfg = unsafe { &(*SYSCFG::ptr()) };
            let exti = unsafe { &(*EXTI::ptr()) };

            /* Enable clock for SYSCFG and USART */
            rcc.apb2enr.modify(|_, w| {
                w.syscfgen().set_bit().usart1en().set_bit()
            });

            /* Enable clock for GPIO Port A, B and F */
            rcc.ahbenr.modify(|_, w| {
                w.iopaen().set_bit().iopben().set_bit().iopfen().set_bit()
            });

            /* Enable clock for I2C1 */
            rcc.apb1enr.modify(|_, w| w.i2c1en().set_bit());

            /* Reset I2C1 */
            rcc.apb1rstr.modify(|_, w| w.i2c1rst().set_bit());
            rcc.apb1rstr.modify(|_, w| w.i2c1rst().clear_bit());

            /* (Re-)configure PB1 for input */
            gpiob.moder.modify(|_, w| unsafe { w.moder1().bits(0) });

            /* Configure pull-down on PB1 */
            gpiob.pupdr.modify(|_, w| unsafe { w.pupdr1().bits(2) });

            /* Enable external interrupt of PB1 */
            syscfg.exticr1.modify(|_, w| unsafe { w.exti1().bits(1) });

            /* Set interrupt request mask for line 1 */
            exti.imr.modify(|_, w| w.mr1().set_bit());

            /* Set interrupt rising trigger for line 1 */
            exti.rtsr.modify(|_, w| w.tr1().set_bit());

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

            /* Enable the I2C processing */
            i2c.cr1.modify(|_, w| w.pe().set_bit());

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

            /* Enable EXTI IRQ, set prio 1 and clear any pending IRQs */
            peripherals.NVIC.enable(Interrupt::EXTI0_1);
            unsafe { peripherals.NVIC.set_priority(Interrupt::EXTI0_1, 1) };
            peripherals.NVIC.clear_pending(Interrupt::EXTI0_1);

            /* Set type of all LEDs to output */
            write_data(i2c, I2C_ADDRESS, &[0x00, 0x00]);

            /* Output a nice message */
            let _ = Write::write_str(
                &mut usart::USARTBuffer(cs),
                "\r\nWelcome to the simple Ixpando 8bit counter. Connect Ixpando to I2C and button against Vcc to PB1. Then push the button a few times.\r\n",
            );
        });
    }
}


interrupt!(EXTI0_1, button_press);


fn button_press() {
    let i2c = unsafe { &(*I2C1::ptr()) };
    let exti = unsafe { &(*EXTI::ptr()) };

    /* A byte array of size 1 to store state in */
    let mut state = [0; 1];

    /* Read the current LED state */
    read_data(i2c, I2C_ADDRESS, 0x12, 1, &mut state);

    /* Write new LED state as previous state + 1 */
    write_data(i2c, I2C_ADDRESS, &[0x12, state[0] + 1]);

    /* Clear interrupt */
    exti.pr.modify(|_, w| w.pr1().set_bit());
}
