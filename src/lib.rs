//! Platform-agnostic MD22 motor driver which uses I2C via
//! [embedded-hal].
//!
//! [embedded-hal]: https://docs.rs/embedded-hal

#![forbid(unsafe_code)]
#![cfg_attr(not(test), no_std)]


mod register;
use embedded_hal as hal;
use crate::register::Register;
use core::fmt::Debug;
use hal::blocking::i2c::{Write, WriteRead};



#[derive(Clone, Copy)]
/// Device I2C bus address switch states
pub enum I2CAddressSwitchState {
    /// Switch 1 - On, Switch 2 - On, Switch 3 - On, Switch 4 - On, 
    OnOnOnOn    = 0xB0,
    /// Switch 1 - Off, Switch 2 - On, Switch 3 - On, Switch 4 - On,
    OffOnOnOn   = 0xB2,
    /// Switch 1 - On, Switch 2 - Off, Switch 3 - On, Switch 4 - On,
    OnOffOnOn   = 0xB4,
    /// Switch 1 - Off, Switch 2 - Off, Switch 3 - On, Switch 4 - On,
    OffOffOnOn  = 0xB6,
    /// Switch 1 - On, Switch 2 - On, Switch 3 - Off, Switch 4 - On,
    OnOnOffOn   = 0xB8,
    /// Switch 1 - Off, Switch 2 - On, Switch 3 - Off, Switch 4 - On,
    OffOnOffOn  = 0xBA,
    /// Switch 1 - On, Switch 2 - Off, Switch 3 - Off, Switch 4 - On,
    OnOffOffOn  = 0xBC,
    /// Switch 1 - Off, Switch 2 - Off, Switch 3 - Off, Switch 4 - On,
    OffOffOffOn = 0xBE,
}

impl I2CAddressSwitchState {
    fn bits(self) -> u8 {
        self as u8
    }
}

#[derive(Clone, Copy)]
/// I2C operating mode
pub enum OperatingMode {
    /// (Default)  The meaning of the speed registers is literal speeds in the range of:  
    /// - 0 (full reverse)
    /// - 128 (stop)
    /// - 255 (full forward)
    Mode0,
    /// The speed registers are interpreted as signed values:  
    /// - -128 (full reverse)
    /// - 0 (stop)
    /// - 127 (full forward)
    Mode1,

}

impl OperatingMode {

    /// Get raw representation of operating mode.
    pub fn bits(self) -> u8 {
        self as u8
    }

    pub fn is_turn_mode(&self) -> bool {
        match self {
            OperatingMode::Mode0 | OperatingMode::Mode1 => false,
            _ => true,
        }
    }
}

/// MD22 Driver
pub struct Md22<I2C> {
    /// Comment above struct member
    i2c: I2C,
    mode: u8,
    address: u8,
}

impl<I2C, E> Md22<I2C>
where
    I2C: WriteRead<Error = E> + Write<Error = E>,
    E: Debug,
{
    /// Create a new MD22 driver from the given I2C peripheral and mode.  
    /// Defaults the speed, acceleration, and turn registers to 0.
    pub fn new(i2c: I2C, mode: OperatingMode, address: I2CAddressSwitchState) -> Result<Self, E> {
        let mut md22 = Md22 {
            i2c: i2c,
            mode: mode.bits(),
            address: address.bits()
        };
        md22.set_mode(mode)?;
        md22.set_acceleration(0)?;
        md22.set_speed(0)?;
        md22.set_turn(0)?;

        Ok(md22)
    }

    /// Set the operating mode.
    pub fn set_mode(&mut self, mode: OperatingMode) -> Result<(), E> {
        let bytes = [Register::Mode.addr(), mode as u8];
        self.i2c.write(self.address, &bytes)?;
        Ok(())
    }

    /// Set the motor speed register to the specified value.
    pub fn set_speed(&mut self, speed: u8) -> Result<(), E> {
        let bytes = [Register::Speed.addr(), speed];
        self.i2c.write(self.address, &bytes)?;
        Ok(())
    }

    /// Set the motor turn register to the specified value.
    pub fn set_turn(&mut self, turn: u8) -> Result<(), E> {
        let bytes = [Register::Turn.addr(), turn];
        self.i2c.write(self.address, &bytes)?;
        Ok(())
    }

    /// Set the motor acceleration register to the specified value.  
    /// The acceleration time is given by this value * 64us * n_steps.
    pub fn set_acceleration(&mut self, acceleration: u8) -> Result<(), E> {
        let bytes = [Register::Acceleration.addr(), acceleration];
        self.i2c.write(self.address, &bytes)?;
        Ok(())
    }
    
    pub fn get_software_revision(&mut self) -> Result<u8, E> {
        let bytes = [Register::SoftwareRevision.addr()];
        let mut buffer: [u8;1] = [0;1];
        self.i2c.write_read(self.address, &bytes, &mut buffer)?;
        Ok(buffer[0])
    }
}


#[cfg(test)]
mod tests {

    
    use crate::*;

    use embedded_hal_mock as hal;

    #[test]
    fn get_software_revision() {
        let address = I2CAddressSwitchState::OnOnOnOn;
        let expectation = [
            hal::i2c::Transaction::write(address.bits(), vec![Register::Mode.addr(), 0]),
            hal::i2c::Transaction::write(address.bits(), vec![Register::Acceleration.addr(), 0]),
            hal::i2c::Transaction::write(address.bits(), vec![Register::Speed.addr(), 0]),
            hal::i2c::Transaction::write(address.bits(), vec![Register::Turn.addr(), 0]),
            hal::i2c::Transaction::write_read(address.bits(), vec![Register::SoftwareRevision.addr()], vec![255]),
        ];
        let i2c = hal::i2c::Mock::new(&expectation);
        let mut md22 = Md22::new(i2c, OperatingMode::Mode0, address).unwrap();
        assert_eq!(255, md22.get_software_revision().unwrap());
    }

    #[test]
    fn set_mode() {
        let address = I2CAddressSwitchState::OnOnOnOn;
        let expectation = [
            hal::i2c::Transaction::write(address.bits(), vec![Register::Mode.addr(), 0]),
            hal::i2c::Transaction::write(address.bits(), vec![Register::Acceleration.addr(), 0]),
            hal::i2c::Transaction::write(address.bits(), vec![Register::Speed.addr(), 0]),
            hal::i2c::Transaction::write(address.bits(), vec![Register::Turn.addr(), 0]),
            hal::i2c::Transaction::write(address.bits(), vec![Register::Mode.addr(), OperatingMode::Mode1.bits()]),
        ];
        let i2c = hal::i2c::Mock::new(&expectation);
        let mut md22 = Md22::new(i2c, OperatingMode::Mode0, address).unwrap();
        md22.set_mode(OperatingMode::Mode1).unwrap();
    }
    
    #[test]
    fn set_acceleration() {
        let address = I2CAddressSwitchState::OnOnOnOn;
        let expectation = [
            hal::i2c::Transaction::write(address.bits(), vec![Register::Mode.addr(), 0]),
            hal::i2c::Transaction::write(address.bits(), vec![Register::Acceleration.addr(), 0]),
            hal::i2c::Transaction::write(address.bits(), vec![Register::Speed.addr(), 0]),
            hal::i2c::Transaction::write(address.bits(), vec![Register::Turn.addr(), 0]),
            hal::i2c::Transaction::write(address.bits(), vec![Register::Acceleration.addr(), 255]),
        ];
        let i2c = hal::i2c::Mock::new(&expectation);
        let mut md22 = Md22::new(i2c, OperatingMode::Mode0, address).unwrap();
        md22.set_acceleration(255).unwrap();
    }

    #[test]
    fn set_speed() {
        let address = I2CAddressSwitchState::OnOnOnOn;
        let expectation = [
            hal::i2c::Transaction::write(address.bits(), vec![Register::Mode.addr(), 0]),
            hal::i2c::Transaction::write(address.bits(), vec![Register::Acceleration.addr(), 0]),
            hal::i2c::Transaction::write(address.bits(), vec![Register::Speed.addr(), 0]),
            hal::i2c::Transaction::write(address.bits(), vec![Register::Turn.addr(), 0]),
            hal::i2c::Transaction::write(address.bits(), vec![Register::Speed.addr(), 255]),
        ];
        let i2c = hal::i2c::Mock::new(&expectation);
        let mut md22 = Md22::new(i2c, OperatingMode::Mode0, address).unwrap();
        md22.set_speed(255).unwrap();
    }

    #[test]
    fn set_turn() {
        let address = I2CAddressSwitchState::OnOnOnOn;
        let expectation = [
            hal::i2c::Transaction::write(address.bits(), vec![Register::Mode.addr(), 0]),
            hal::i2c::Transaction::write(address.bits(), vec![Register::Acceleration.addr(), 0]),
            hal::i2c::Transaction::write(address.bits(), vec![Register::Speed.addr(), 0]),
            hal::i2c::Transaction::write(address.bits(), vec![Register::Turn.addr(), 0]),
            hal::i2c::Transaction::write(address.bits(), vec![Register::Turn.addr(), 255]),
        ];
        let i2c = hal::i2c::Mock::new(&expectation);
        let mut md22 = Md22::new(i2c, OperatingMode::Mode0, address).unwrap();
        md22.set_turn(255).unwrap();
    }
}
