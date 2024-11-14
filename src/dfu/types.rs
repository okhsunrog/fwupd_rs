use bytes::BytesMut;

#[derive(Debug, Clone, Copy)]
pub enum Command {
    ReadBootloaderInfo = 0,
    ReadProgramCrc = 3,
    BootloaderQuit = 5,
    WriteProgramMemory = 6,
}

#[repr(C, packed)]
pub struct FirmwareMetadata {
    pub image_size: u32,
    pub image_crc: u32,
}

#[repr(C, packed)]
pub struct Region {
    pub count: u32,
    pub size: u32,
}

#[repr(C, packed)]
pub struct DeviceId {
    pub id: u16,
    pub rev: u16,
    pub uid: [u8; 16],
}

#[repr(C, packed)]
pub struct DeviceMemoryMap {
    pub metadata_address: u32,
    pub metadata_size: u32,
    pub firmware_address: u32,
    pub firmware_size: u32,
    pub flash_address: u32,
    pub flash_size: u32,
    pub flash_write_blocksize: u16,
    pub regions: [Region; 5],
}

#[repr(C, packed)]
pub struct InfoBlockV2 {
    pub version: u8,
    pub max_block_size: u16,
    pub device: DeviceId,
    pub unused: [u8; 18],
    pub memmap: DeviceMemoryMap,
}

pub struct DfuConfig {
    pub uri: String,
    pub filename: Option<String>,
    pub block_size: usize,
    pub get_info: bool,
    pub update: bool,
    pub overwrite: bool,
    pub verify: bool,
    pub quit: bool,
    pub dev_netid: usize,
    pub dev_speed: usize,
    pub upd_speed: usize,
    pub lnk_speed: usize,
    pub upd_mode: UpdateMode,
    pub gap_filling: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UpdateMode {
    None = 0,
    Direct = 1,
    Link = 2,
}
