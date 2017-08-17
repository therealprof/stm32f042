use USART1;

pub fn read_char(usart1: &USART1, echo: bool) -> u8 {
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
    if echo && c != 0 {
        /* Wait until the USART is clear to send */
        while usart1.isr.read().txe().bit_is_clear() {}

        /* Write the current character to the output register */
        usart1.tdr.modify(|_, w| unsafe { w.bits(c) });
    }

    c as u8
}
