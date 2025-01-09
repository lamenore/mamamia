use std::path::PathBuf;

use num_enum::FromPrimitive;

use super::room;

pub struct Rom {
    data: Vec<u8>,
}

impl Rom {
    pub fn open(path: PathBuf) -> Self {
        let data = std::fs::read(path).unwrap();
        Self { data }
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
    pub room_id: u16,
    pub bitflag: u8,
    direction: u8,
    pub door_cap_x: u8,
    pub door_cap_y: u8,
    pub screen_x: u8,
    pub screen_y: u8,
    pub distance_to_spawn: u16,
    door_asm_pointer: u16,
}

impl DoorHeader {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let room_id = u16::from_le_bytes([bytes[0], bytes[1]]);
        if room_id == 0 {
            Self {
                ..Default::default()
            }
        } else {
            Self {
                room_id,
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
        match self.direction & 0b0000_0011 {
            0 => DoorDirection::Right,
            1 => DoorDirection::Left,
            2 => DoorDirection::Down,
            3 => DoorDirection::Up,
            _ => unreachable!("Invalid door direction"),
        }
    }

    pub fn should_close_behind(&self) -> bool {
        (self.direction & 0b0000_0100) != 0
    }
}
