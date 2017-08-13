use I2C1;

pub fn write_data(i2c: &I2C1, addr: u8, data: &[u8]) -> Option<()> {
    /* Wait while someone else is using the I2C bus, just to be on the sure side */
    while i2c.cr2.read().start().bit_is_set() {}

    /* Set up current address, we're trying a "read" command and not going to set anything
     * and make sure we end a non-NACKed read (i.e. if we found a device) properly */
    i2c.cr2.modify(|_, w| unsafe {
        w.sadd1()
            .bits(addr)
            .nbytes()
            .bits((data.len()) as u8)
            .rd_wrn()
            .clear_bit()
            .autoend()
            .set_bit()
    });

    /* Send a START condition */
    i2c.cr2.modify(|_, w| w.start().set_bit());

    for c in data {
        /* Push out a byte of data */
        i2c.txdr.write(|w| unsafe { w.bits(*c as u32) });

        /* Wait until the transmit buffer is empty and there hasn't been either a NACK or STOP
         * being received */
        let mut isr;
        while {
            isr = i2c.isr.read();
            isr.txis().bit_is_clear() && isr.nackf().bit_is_clear() &&
                isr.stopf().bit_is_clear() && isr.tc().bit_is_clear()
        }
        {}

        /* If we received a NACK, then this is an error */
        if isr.nackf().bit_is_set() {
            i2c.icr.write(|w| w.stopcf().set_bit().nackcf().set_bit());
            return None;
        }
    }

    /* Fallthrough is success */
    i2c.icr.write(|w| w.stopcf().set_bit().nackcf().set_bit());
    Some(())
}
