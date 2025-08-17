use std::fmt::UpperHex;

#[derive(Debug, Copy, Clone)]
pub(crate) struct LoRom {
    addrlo: u8,
    addrhi: u8,
    bank: u8,
}

impl LoRom {
    /// Converts a 24-bit hexadecimal SNES LoROM address into a `LoRom` struct.
    ///
    /// The input `hex` is expected to be in the format "BBHHLL", where:
    /// - `BB` is the bank byte,
    /// - `HH` is the high byte of the address,
    /// - `LL` is the low byte of the address.
    pub(crate) fn from_hex(hex: u32) -> Result<Self, anyhow::Error> {
        let bank = (hex >> 16) as u8;
        let addrhi = ((hex >> 8) & 0xFF) as u8;
        let addrlo = (hex & 0xFF) as u8;

        Ok(LoRom {
            addrlo,
            addrhi,
            bank,
        })
    }

    #[allow(dead_code)]
    pub(crate) fn to_hex(self) -> u32 {
        (self.bank as u32) << 16 | (self.addrhi as u32) << 8 | (self.addrlo as u32)
    }

    pub(crate) fn to_pc(self) -> PC {
        PC((self.addrlo as u32)
            + (256 * (self.addrhi as u32))
            + (32768 * (self.bank as u32 & 0x7F))
            - 32256
            - 512)
    }
}

impl UpperHex for LoRom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02X}{:02X}{:02X}", self.bank, self.addrhi, self.addrlo)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PC(pub(crate) u32);

impl PC {
    #[allow(dead_code)]
    /// Converts a `PC` address to a `LoRom` address. The input `PC` address is expected to be a
    /// valid PC address for a LoROM SNES game. Note that the address is adjusted by 512 to account
    /// for the header in LoROM games.
    pub(crate) fn to_lorom(self) -> LoRom {
        assert!(self.0 < 4194304, "Invalid PC address!");

        LoRom {
            addrlo: ((self.0 + 512) & 0xFF) as u8,
            addrhi: (((self.0 + 512) >> 8) & 0xFF) as u8,
            bank: (((self.0 + 512) >> 16) & 0xFF) as u8,
        }
    }
}

impl UpperHex for PC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:08X}", self.0)
    }
}
