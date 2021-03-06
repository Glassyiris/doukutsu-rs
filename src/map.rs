use std::io;
use std::io::{Error, ErrorKind};

use byteorder::{LE, ReadBytesExt};

use crate::ggez::GameError::ResourceLoadError;
use crate::ggez::GameResult;
use crate::str;

static SUPPORTED_PXM_VERSIONS: [u8; 1] = [0x10];
static SUPPORTED_PXE_VERSIONS: [u8; 2] = [0, 0x10];

pub struct Map {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<u8>,
    pub attrib: [u8; 0x100],
}

impl Map {
    pub fn load_from<R: io::Read>(mut map_data: R, mut attrib_data: R) -> GameResult<Map> {
        let mut magic = [0; 3];

        map_data.read_exact(&mut magic)?;

        if &magic != b"PXM" {
            return Err(ResourceLoadError(str!("Invalid magic")));
        }

        let version = map_data.read_u8()?;

        // It's something Booster's Lab supports but I haven't seen anywhere being used in practice
        if !SUPPORTED_PXM_VERSIONS.contains(&version) {
            return Err(ResourceLoadError(format!("Unsupported PXM version: {:#x}", version)));
        }

        let width = map_data.read_u16::<LE>()? as usize;
        let height = map_data.read_u16::<LE>()? as usize;
        let mut tiles = vec![0u8; width * height];
        let mut attrib = [0u8; 0x100];

        log::info!("Map size: {}x{}", width, height);

        map_data.read_exact(&mut tiles)?;
        if attrib_data.read_exact(&mut attrib).is_err() {
            log::warn!("Map attribute data is shorter than 256 bytes!");
        }

        Ok(Map {
            width,
            height,
            tiles,
            attrib,
        })
    }

    pub fn get_attribute(&self, x: usize, y: usize) -> u8 {
        self.attrib[*self.tiles.get(self.width * y + x).unwrap_or_else(|| &0u8) as usize]
    }
}

#[derive(Debug)]
pub struct NPCData {
    pub id: u16,
    pub x: i16,
    pub y: i16,
    pub flag_num: u16,
    pub event_num: u16,
    pub npc_type: u16,
    pub flags: u16,
    pub layer: u8,
}

impl NPCData {
    pub fn load_from<R: io::Read>(mut data: R) -> GameResult<Vec<NPCData>> {
        let mut magic = [0; 3];

        data.read_exact(&mut magic)?;

        if &magic != b"PXE" {
            return Err(ResourceLoadError(str!("Invalid magic")));
        }

        let version = data.read_u8()?;
        if !SUPPORTED_PXE_VERSIONS.contains(&version) {
            return Err(ResourceLoadError(format!("Unsupported PXE version: {:#x}", version)));
        }

        let count = data.read_u32::<LE>()? as usize;
        let mut npcs = Vec::with_capacity(count);

        for i in 0..count {
            let x = data.read_i16::<LE>()?;
            let y = data.read_i16::<LE>()?;
            let flag_num = data.read_u16::<LE>()?;
            let event_num = data.read_u16::<LE>()?;
            let npc_type = data.read_u16::<LE>()?;
            let flags = data.read_u16::<LE>()?;

            // booster's lab also specifies a layer field in version 0x10, prob for multi-layered maps
            let layer = if version == 0x10 { data.read_u8()? } else { 0 };

            npcs.push(NPCData {
                id: 170 + i as u16,
                x,
                y,
                flag_num,
                event_num,
                npc_type,
                flags,
                layer,
            })
        }

        Ok(npcs)
    }
}
