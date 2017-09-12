#![feature(used)]
#![feature(const_fn)]
#![no_std]

extern crate cortex_m;

#[macro_use(interrupt)]
extern crate stm32f042;

use stm32f042::*;

use stm32f042::Interrupt;

static mut QEI_STATE: u8 = 0;
static mut RGB: [u8; 3] = [0; 3];
static mut COLOR: usize = 0;


fn main() {
    cortex_m::interrupt::free(|cs| {
        let rcc = RCC.borrow(cs);
        let gpioa = GPIOA.borrow(cs);
        let gpiob = GPIOB.borrow(cs);
        let tim2 = TIM2.borrow(cs);
        let exti = EXTI.borrow(cs);
        let syscfg = SYSCFG.borrow(cs);
        let nvic = NVIC.borrow(cs);

        /* Enable clock for SYSCFG */
        rcc.apb2enr.modify(|_, w| w.syscfgen().set_bit());

        /* Enable clock for GPIO Port A and B */
        rcc.ahbenr.modify(
            |_, w| w.iopaen().set_bit().iopben().set_bit(),
        );

        /* Enable clock for TIM2 */
        rcc.apb1enr.modify(|_, w| w.tim2en().set_bit());

        /* (Re-)configure PB1 for input */
        gpiob.moder.modify(|_, w| unsafe { w.moder1().bits(0) });

        /* Configure pull-down on PB1 */
        gpiob.pupdr.modify(|_, w| unsafe { w.pupdr1().bits(2) });

        /* Enable external interrupt of PB1 */
        syscfg.exticr1.modify(|_, w| unsafe { w.exti1().bits(1) });

        /* (Re-)configure PA4 and PA5 for input */
        gpioa.moder.modify(|_, w| unsafe {
            w.moder4().bits(0).moder5().bits(0)
        });

        /* Configure pull-down on PA4 and PA5 */
        gpioa.pupdr.modify(|_, w| unsafe {
            w.pupdr4().bits(2).pupdr5().bits(2)
        });

        /* Set interrupt request mask for lines 1, 4 and 5*/
        exti.imr.write(|w| {
            w.mr1().set_bit().mr4().set_bit().mr5().set_bit()
        });

        /* Set interrupt rising trigger for lines 1, 4 and 5 */
        exti.rtsr.modify(|_, w| {
            w.tr1().set_bit().tr4().set_bit().tr5().set_bit()
        });

        /* Set interrupt falling trigger for 4 and 5 */
        exti.ftsr.modify(|_, w| w.tr4().set_bit().tr5().set_bit());

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

        /* Enable EXTI IRQs, set prio 1 and clear any pending IRQs */
        nvic.enable(Interrupt::EXTI0_1);
        unsafe { nvic.set_priority(Interrupt::EXTI0_1, 1) };
        nvic.clear_pending(Interrupt::EXTI0_1);

        nvic.enable(Interrupt::EXTI4_15);
        unsafe { nvic.set_priority(Interrupt::EXTI4_15, 1) };
        nvic.clear_pending(Interrupt::EXTI4_15);

        /* Clear interrupt lines 1, 4 and 5 */
        exti.pr.modify(|_, w| {
            w.pr1().set_bit().pr4().set_bit().pr5().set_bit()
        });
    });
}


/* Set PWM channel compare register to the specified intensities */
fn change_color(tim2: &stm32f042::TIM2, r: u8, g: u8, b: u8) {
    tim2.ccr2.modify(|_, w| unsafe { w.bits(u32::from(r)) });
    tim2.ccr3.modify(|_, w| unsafe { w.bits(u32::from(g)) });
    tim2.ccr4.modify(|_, w| unsafe { w.bits(u32::from(b)) });
}


interrupt!(EXTI0_1, button_press);
interrupt!(EXTI4_15, button_press);


/* Handle button press or rotation of rotary encoder */
fn button_press() {
    cortex_m::interrupt::free(|cs| {
        let tim2 = TIM2.borrow(cs);
        let exti = EXTI.borrow(cs);
        let gpioa = GPIOA.borrow(cs);

        let qei_state;

        /* Read out current state of QEI inputs A and B */
        let idr = gpioa.idr.read();
        let a = idr.idr5().bit_is_set();
        let b = idr.idr4().bit_is_set();

        /* Move old state in state variable to the left and add new state */
        unsafe {
            QEI_STATE = (QEI_STATE & 3) << 2 | ((a as u8) << 1) | b as u8;
            qei_state = QEI_STATE;
        }

        /* Check wheter the interrupt was caused by button press */
        if exti.pr.read().pr1().bit_is_set() {
            unsafe {
                /* If so, switch controlled color component */
                COLOR += 1;
                if COLOR > 2 {
                    COLOR = 0
                }
            };
        }

        /* Check whether old state -> new state change makes sense */
        let dir = match qei_state & 0xf {
            /* State changed clock-wise => we're going up */
            0b0001 | 0b0111 | 0b1110 | 0b1000 => 1,

            /* State changed counter clock-wise => we're going up */
            0b0100 | 0b1101 | 0b1011 | 0b0010 => -1,

            /* Missed a change or false reading => do nothing */
            _ => 0,
        };

        /* Apply rotary change to current color component */
        if dir == 1 {
            unsafe {
                if RGB[COLOR] <= 254 {
                    RGB[COLOR] += 1
                };
                change_color(tim2, RGB[0], RGB[1], RGB[2]);
            }
        } else if dir == -1 {
            unsafe {
                if RGB[COLOR] >= 1 {
                    RGB[COLOR] -= 1
                };
                change_color(tim2, RGB[0], RGB[1], RGB[2]);
            }
        }

        /* Clear interrupt lines 1, 4 and 5 */
        exti.pr.modify(|_, w| {
            w.pr1().set_bit().pr4().set_bit().pr5().set_bit()
        });
    });
}
