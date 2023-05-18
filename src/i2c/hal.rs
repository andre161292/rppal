use embedded_hal::i2c::{self, ErrorType, I2c as I2cHal, Operation as I2cOperation, Operation};

use super::{Error, I2c};

/// `Write` trait implementation for `embedded-hal` v0.2.7.
impl embedded_hal_0::blocking::i2c::Write for I2c {
    type Error = Error;

    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        I2cHal::write(self, address, bytes)
    }
}

/// `Read` trait implementation for `embedded-hal` v0.2.7.
impl embedded_hal_0::blocking::i2c::Read for I2c {
    type Error = Error;

    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        I2cHal::read(self, address, buffer)
    }
}

/// `WriteRead` trait implementation for `embedded-hal` v0.2.7.
impl embedded_hal_0::blocking::i2c::WriteRead for I2c {
    type Error = Error;

    fn write_read(
        &mut self,
        address: u8,
        bytes: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        I2cHal::write_read(self, address, bytes, buffer)
    }
}

impl ErrorType for I2c {
    type Error = Error;
}

impl i2c::Error for Error {
    fn kind(&self) -> i2c::ErrorKind {
        if let Error::Io(e) = self {
            use std::io::ErrorKind::*;

            match e.kind() {
                /* ResourceBusy | */ InvalidData => i2c::ErrorKind::Bus,
                WouldBlock => i2c::ErrorKind::ArbitrationLoss,
                _ => i2c::ErrorKind::Other,
            }
        } else {
            i2c::ErrorKind::Other
        }
    }
}

/// `I2c` trait implementation for `embedded-hal` v1.0.0-alpha.9.
impl I2cHal for I2c {
    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        self.set_slave_address(u16::from(address))?;
        I2c::write(self, bytes)?;

        Ok(())
    }

    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.set_slave_address(u16::from(address))?;
        I2c::read(self, buffer)?;

        Ok(())
    }

    fn write_iter<B>(&mut self, address: u8, bytes: B) -> Result<(), Self::Error>
    where
        B: IntoIterator<Item = u8>,
    {
        let bytes: Vec<_> = bytes.into_iter().collect();
        I2cHal::write(self, address, &bytes)
    }

    fn write_read(
        &mut self,
        address: u8,
        bytes: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.set_slave_address(u16::from(address))?;
        I2c::write_read(self, bytes, buffer)?;

        Ok(())
    }

    fn write_iter_read<B>(
        &mut self,
        address: u8,
        bytes: B,
        buffer: &mut [u8],
    ) -> Result<(), Self::Error>
    where
        B: IntoIterator<Item = u8>,
    {
        let bytes: Vec<_> = bytes.into_iter().collect();
        self.transaction(
            address,
            &mut [I2cOperation::Write(&bytes), I2cOperation::Read(buffer)],
        )
    }

    fn transaction(
        &mut self,
        address: u8,
        operations: &mut [I2cOperation],
    ) -> Result<(), Self::Error> {
        let address = u16::from(address);
        self.set_slave_address(address)?;

        let mut last_operation: Option<&Operation> = None;
        for operation in operations {
            if let Some(last_op) = last_operation {
                if std::mem::discriminant(last_op) != std::mem::discriminant(operation) {
                    self.set_slave_address(address)?;
                }
            }

            match operation {
                Operation::Read(buffer) => {
                    let buffer_len = buffer.len();
                    let mut buf = vec![0; 1 + buffer_len].into_boxed_slice();
                    let _read_bytes = self.read(&mut buf)?;
                    if buf[0] != 1 {
                        // TODO Throw correct error
                        return Err(Error::FeatureNotSupported);
                    }
                    buffer.copy_from_slice(&buf[1..=buffer_len]);
                }
                Operation::Write(_data) => {
                    // FIXME Untested, so throw.
                    // self.write(data)?;
                    unimplemented!();
                }
            }

            last_operation = Some(operation)
        }

        Ok(())
    }

    fn transaction_iter<'a, O>(&mut self, address: u8, operations: O) -> Result<(), Self::Error>
    where
        O: IntoIterator<Item = I2cOperation<'a>>,
    {
        let mut ops: Vec<_> = operations.into_iter().collect();
        self.transaction(address, &mut ops)
    }
}
