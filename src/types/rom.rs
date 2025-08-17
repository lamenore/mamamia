use std::path::PathBuf;

use num_enum::FromPrimitive;

pub struct Rom {
    data: Vec<u8>,
}

impl Rom {
    pub fn open(path: PathBuf) -> Self {
        let data = std::fs::read(path).unwrap();
        Self { data }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}

impl<Idx> std::ops::Index<Idx> for Rom
where
    Idx: std::slice::SliceIndex<[u8]>,
{
    type Output = <Idx as std::slice::SliceIndex<[u8]>>::Output;

    fn index(&self, index: Idx) -> &Self::Output {
        &self.data[index]
    }
}

#[derive(Debug, FromPrimitive, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum DoorDirection {
    #[default]
    Right,
    Left,
    Down,
    Up,
}

impl std::fmt::Display for DoorDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            DoorDirection::Right => write!(f, "Right"),
            DoorDirection::Left => write!(f, "Left"),
            DoorDirection::Down => write!(f, "Down"),
            DoorDirection::Up => write!(f, "Up"),
        }
    }
}

#[derive(Default, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct DoorHeader {
    /// Room ID to which the door leads
    pub troom_id: u16,
    /// For entering a different area, the bitflag will always be 0x40,
    /// 80 = Elevator is leading into a room that's in the same area that Samus is currently in. C0 = Elevator leads to a different area.
    pub bitflag: u8,
    /// No door closes behind Samus: 00 = right, 01 = left, 02 = down, 03 = up. Door closes behind Samus: 04 = right, 05 = left, 06 = down, 07 = up.
    direction: u8,
    /// Horizontal position of the closing blue door cap in the next room, counted in tiles.
    pub door_cap_x: u8,
    /// Vertical position of the closing blue door cap in the next room, counted in tiles.
    pub door_cap_y: u8,
    /// Horizontal position, counted from the very left in screens.
    pub screen_x: u8,
    /// Vertical position, counted from the very top in screens.
    pub screen_y: u8,
    /// Left/right doors use 8000 by default. For doors leading up, use 01C0, and for doors leading down, 0140 is good.
    pub distance_to_spawn: u16,
    ///  0000 by default, but can point to custom code in bank $8F. Used sometimes to change the scroll color for certain screens as the room loads.
    door_asm_pointer: u16,
}

impl DoorHeader {
    pub const SIZE: usize = 12;

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let room_id = u16::from_le_bytes([bytes[0], bytes[1]]);
        if room_id == 0 {
            Self {
                ..Default::default()
            }
        } else {
            Self {
                troom_id: room_id,
                bitflag: bytes[2],
                direction: bytes[3],
                door_cap_x: bytes[4],
                door_cap_y: bytes[5],
                screen_x: bytes[6],
                screen_y: bytes[7],
                distance_to_spawn: u16::from_le_bytes([bytes[8], bytes[9]]),
                door_asm_pointer: u16::from_le_bytes([bytes[10], bytes[11]]),
            }
        }
    }

    pub fn get_door_direction(&self) -> DoorDirection {
        match self.direction & 0b11 {
            0 => DoorDirection::Right,
            1 => DoorDirection::Left,
            2 => DoorDirection::Down,
            3 => DoorDirection::Up,
            _ => unreachable!("Invalid door direction"),
        }
    }

    #[allow(dead_code)]
    pub fn should_close_behind(&self) -> bool {
        (self.direction & 0b0000_0100) != 0
    }

    pub fn is_elevator(&self) -> bool {
        (self.bitflag & 0x80) != 0
    }

    #[allow(dead_code)]
    pub fn is_same_area(&self) -> bool {
        (self.bitflag & 0x40) != 0
    }
}
