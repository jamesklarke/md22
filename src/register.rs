#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Register {
    /// MD22 Mode Register
    Mode = 0x00,
    /// MD22 Speed Register
    Speed = 0x01,
    /// MD22 Turn Register
    Turn = 0x02,
    /// MD22 Acceleration Register
    Acceleration = 0x03,
    /// MD22 Software Revision Register
    SoftwareRevision = 0x07,
}


impl Register {
    /// Get the register address.
    pub fn addr(self) -> u8 {
        self as u8
    }

    pub fn is_read_only(self) -> bool {
        match self {
            Register::SoftwareRevision => true,
            _ => false,
        }
    }
}