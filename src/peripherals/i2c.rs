use I2C1;

pub fn write_data(i2c: &I2C1, addr: u8, data: &[u8]) -> Option<()> {
    /* Set up current address, we're trying a "read" command and not going to set anything
     * and make sure we end a non-NACKed read (i.e. if we found a device) properly */
    i2c.cr2.modify(|_, w| unsafe {
        w.sadd1()
            .bits(addr)
            .nbytes()
            .bits(data.len() as u8)
            .rd_wrn()
            .clear_bit()
            .autoend()
            .set_bit()
    });

    /* Send a START condition */
    i2c.cr2.modify(|_, w| w.start().set_bit());

    for c in data {
        /* Wait until we're ready for sending */
        while i2c.isr.read().txis().bit_is_clear() {}

        /* Push out a byte of data */
        i2c.txdr.write(|w| unsafe { w.bits(u32::from(*c)) });

        /* If we received a NACK, then this is an error */
        if i2c.isr.read().nackf().bit_is_set() {
            i2c.icr.write(|w| w.stopcf().set_bit().nackcf().set_bit());
            return None;
        }
    }

    /* Fallthrough is success */
    i2c.icr.write(|w| w.stopcf().set_bit().nackcf().set_bit());
    Some(())
}


pub fn read_data(i2c: &I2C1, addr: u8, req: u8, size: u8, data: &mut [u8]) -> Option<()> {
    /* Set up current address, we're trying a "read" command and not going to set anything
     * and make sure we end a non-NACKed read (i.e. if we found a device) properly */
    i2c.cr2.modify(|_, w| unsafe {
        w.sadd1()
            .bits(addr)
            .nbytes()
            .bits(1)
            .rd_wrn()
            .clear_bit()
            .autoend()
            .clear_bit()
    });

    /* Send a START condition */
    i2c.cr2.modify(|_, w| w.start().set_bit());

    /* Wait until the transmit buffer is empty and there hasn't been either a NACK or STOP
     * being received */
    let mut isr;
    while {
        isr = i2c.isr.read();
        isr.txis().bit_is_clear() && isr.nackf().bit_is_clear() && isr.stopf().bit_is_clear() &&
            isr.tc().bit_is_clear()
    }
    {}

    /* If we received a NACK, then this is an error */
    if isr.nackf().bit_is_set() {
        i2c.icr.write(|w| w.stopcf().set_bit().nackcf().set_bit());
        return None;
    }

    /* Push out a byte of data */
    i2c.txdr.write(|w| unsafe { w.bits(u32::from(req)) });

    /* Wait until data was sent */
    while i2c.isr.read().tc().bit_is_clear() {}

    /* Set up current address, we're trying a "read" command and not going to set anything
     * and make sure we end a non-NACKed read (i.e. if we found a device) properly */
    i2c.cr2.modify(|_, w| unsafe {
        w.sadd1().bits(addr).nbytes().bits(size).rd_wrn().set_bit()
    });

    /* Send a START condition */
    i2c.cr2.modify(|_, w| w.start().set_bit());

    /* Send the autoend after setting the start to get a restart */
    i2c.cr2.modify(|_, w| w.autoend().set_bit());

    /* Read in all bytes */
    for c in data.iter_mut() {
        while i2c.isr.read().rxne().bit_is_clear() {}
        let value = i2c.rxdr.read().bits() as u8;
        *c = value;
    }

    /* Clear flags if they somehow ended up set */
    i2c.icr.write(|w| w.stopcf().set_bit().nackcf().set_bit());

    Some(())
}
