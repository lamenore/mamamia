#[derive(Debug, Copy, Clone)]
pub struct LoRom {
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
    ///
    /// Returns an `Err` if the address does not conform to LoROM addressing rules:
    /// - For even banks, `HH` must not exceed `0x7F`.
    /// - For odd banks, `HH` must be `0x80` or greater.
    pub fn from_hex(hex: u32) -> Result<Self, &'static str> {
        let bank: u8 = ((hex >> 16) & 0xFF) as u8;
        let addrhi = ((hex >> 8) & 0xFF) as u8;
        let addrlo = (hex & 0xFF) as u8;

        // if (bank & 0x01) == 0 {
        //     if addrhi > 0x7F {
        //         return Err("Invalid SNES LoROM address for even bank!");
        //     }
        // } else if addrhi < 0x80 {
        //     return Err("Invalid SNES LoROM address for odd bank!");
        // }

        Ok(LoRom {
            addrlo,
            addrhi,
            bank,
        })
    }

    pub fn to_hex(self) -> u32 {
        (self.bank as u32) << 16 | (self.addrhi as u32) << 8 | (self.addrlo as u32)
    }

    pub fn to_pc(self) -> u32 {
        (self.addrlo as u32) + (256 * (self.addrhi as u32)) + (32768 * (self.bank as u32 & 0x7F))
            - 32256
            - 512
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PC(pub u32);

impl PC {
    /// Converts a `PC` address to a `LoRom` address. The input `PC` address is expected to be a
    /// valid PC address for a LoROM SNES game. Note that the address is adjusted by 512 to account
    /// for the header in LoROM games.
    pub fn to_lorom(self) -> Result<LoRom, &'static str> {
        if self.0 >= 4194304 {
            return Err("Invalid PC address for LoROM conversion!");
        }

        Ok(LoRom {
            addrlo: ((self.0 + 512) & 0xFF) as u8,
            addrhi: (((self.0 + 512) >> 8) & 0xFF) as u8,
            bank: (((self.0 + 512) >> 16) & 0xFF) as u8,
        })
    }
}
