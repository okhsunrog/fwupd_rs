use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::time::sleep;
use tokio_stream::StreamExt;
use bytes::BytesMut;
use log::{info, error, warn};

use crate::protocols::{apl, lpl};
use crate::error::Result;

mod config;
mod types;

pub use config::*;
pub use types::*;

const MAX_RECONNECTION_ATTEMPTS: usize = 3;

pub struct DfuStream<T> {
    stream: T,
    config: DfuConfig,
    lpl: lpl::LplStream,
    apl: apl::AplStream,
    buffer: BytesMut,
}

impl<T: AsyncRead + AsyncWrite + Unpin> DfuStream<T> {
    pub fn new(stream: T, config: DfuConfig) -> Result<Self> {
        config.validate()?;
        
        let (apl, apl_tx) = apl::AplStream::new(1024);
        let (lpl, _) = lpl::LplStream::new(1024, apl_tx);

        Ok(Self {
            stream,
            config,
            lpl,
            apl,
            buffer: BytesMut::with_capacity(1024),
        })
    }

    pub async fn update(&mut self) -> Result<()> {
        info!("Starting firmware update process");

        if self.config.upd_mode != UpdateMode::None {
            self.auto_enter().await?;
        }

        if self.config.get_info || self.config.update || self.config.verify {
            let info = self.read_bootloader_info().await?;
            self.log_device_info(&info);

            if self.config.update || self.config.verify {
                self.process_firmware(&info).await?;
            }
        }

        if self.config.quit {
            self.quit_bootloader().await?;
        }

        if self.config.upd_mode != UpdateMode::None {
            self.auto_exit().await?;
        }

        info!("Firmware update completed successfully");
        Ok(())
    }

    async fn auto_enter(&mut self) -> Result<()> {
        info!("Entering bootloader mode");
        
        // Set initial speed
        self.set_speed(self.config.lnk_speed).await?;

        // Try to detect bootloader
        if self.detect_bootloader().await.is_ok() {
            info!("Bootloader detected");
            return Ok(());
        }

        // Send reboot command
        self.send_reboot_command().await?;
        
        // Wait for bootloader
        sleep(Duration::from_millis(1000)).await;
        
        // Verify bootloader is active
        self.detect_bootloader().await?;
        
        info!("Successfully entered bootloader mode");
        Ok(())
    }

    async fn process_firmware(&mut self, info: &InfoBlockV2) -> Result<()> {
        if self.config.update {
            info!("Starting firmware update");
            self.write_firmware(info).await?;
        }

        if self.config.verify {
            info!("Verifying firmware");
            self.verify_firmware(info).await?;
        }

        Ok(())
    }

    async fn quit_bootloader(&mut self) -> Result<()> {
        info!("Exiting bootloader mode");
        self.lpl.send_request(
            &mut self.stream,
            apl::AplRequestType::WriteRequest,
            0,
            0,
            Command::BootloaderQuit as usize,
            0,
            0,
        ).await?;
        Ok(())
    }

    async fn auto_exit(&mut self) -> Result<()> {
        info!("Restoring normal operation mode");
        self.set_speed(self.config.lnk_speed).await?;
        Ok(())
    }

    async fn set_speed(&mut self, speed: usize) -> Result<()> {
        // Implementation depends on stream type
        Ok(())
    }

    fn log_device_info(&self, info: &InfoBlockV2) {
        info!("Device Information:");
        info!("  Version: {:#04x}", info.version);
        info!("  Device ID: {:#06x}", info.device.id);
        info!("  Revision: {:#06x}", info.device.rev);
    }
}

impl<T: AsyncRead + AsyncWrite + Unpin> DfuStream<T> {
    async fn write_firmware(&mut self, info: &InfoBlockV2) -> Result<()> {
        let firmware = self.load_firmware()?;
        
        // Check if firmware is already installed
        let current_crc = self.read_firmware_crc(
            info.memmap.firmware_address,
            info.memmap.firmware_size
        ).await?;
        
        let new_crc = calculate_crc32(&firmware);
        if current_crc == new_crc && !self.config.overwrite {
            info!("Firmware already up to date (CRC: {:#010x})", new_crc);
            return Ok(());
        }

        // Write firmware in blocks
        let block_size = self.config.block_size.min(info.max_block_size as usize);
        let total_blocks = (firmware.len() + block_size - 1) / block_size;

        for (i, chunk) in firmware.chunks(block_size).enumerate() {
            let offset = i * block_size;
            self.write_block(
                chunk,
                info.memmap.firmware_address + offset as u32
            ).await?;

            let progress = ((i + 1) * 100) / total_blocks;
            info!("Progress: {}%", progress);
        }

        Ok(())
    }

    async fn verify_firmware(&mut self, info: &InfoBlockV2) -> Result<()> {
        let firmware = self.load_firmware()?;
        let firmware_crc = calculate_crc32(&firmware);

        let device_crc = self.read_firmware_crc(
            info.memmap.firmware_address,
            info.memmap.firmware_size
        ).await?;

        if firmware_crc != device_crc {
            error!("Verification failed: CRC mismatch");
            error!("Expected: {:#010x}, Got: {:#010x}", firmware_crc, device_crc);
            return Err(Error::VerificationFailed);
        }

        info!("Firmware verification successful");
        Ok(())
    }

    async fn write_block(&mut self, data: &[u8], address: u32) -> Result<()> {
        self.lpl.send_request(
            &mut self.stream,
            apl::AplRequestType::WriteRequest,
            data.len(),
            0,
            Command::WriteProgramMemory as usize,
            address as usize,
            data.len(),
        ).await?;

        Ok(())
    }

    async fn read_firmware_crc(&mut self, address: u32, size: u32) -> Result<u32> {
        self.lpl.send_request(
            &mut self.stream,
            apl::AplRequestType::ReadRequest,
            size_of::<u32>(),
            0,
            Command::ReadProgramCrc as usize,
            address as usize,
            size as usize,
        ).await?;

        // Read CRC response
        let mut crc = [0u8; 4];
        self.stream.read_exact(&mut crc).await?;
        Ok(u32::from_le_bytes(crc))
    }
}

use crc::{Crc, CRC_32_ISO_HDLC};

fn calculate_crc32(data: &[u8]) -> u32 {
    let crc = Crc::<u32>::new(&CRC_32_ISO_HDLC);
    crc.checksum(data)
}

impl<T: AsyncRead + AsyncWrite + Unpin> DfuStream<T> {
    fn load_firmware(&self) -> Result<Vec<u8>> {
        let filename = self.config.filename.as_ref()
            .ok_or(Error::NoFirmwareFile)?;
            
        let ihex = ihex::Reader::new(filename)
            .map_err(Error::HexFileError)?;
            
        let mut firmware = vec![0xFF; self.max_firmware_size()];
        
        for record in ihex {
            let record = record.map_err(Error::HexFileError)?;
            if let ihex::Record::Data { offset, value } = record {
                firmware[offset..offset + value.len()]
                    .copy_from_slice(&value);
            }
        }
        
        Ok(firmware)
    }
}

impl<T: AsyncRead + AsyncWrite + Unpin> DfuStream<T> {
    fn max_firmware_size(&self) -> usize {
        // Default to 1MB if not specified in config
        1024 * 1024
    }

    fn validate_firmware(&self, firmware: &[u8], info: &InfoBlockV2) -> Result<()> {
        // Check firmware size
        if firmware.len() > info.memmap.firmware_size as usize {
            return Err(Error::FirmwareTooLarge);
        }

        // Verify device ID if needed
        if info.version >= 0x30 && !self.config.overwrite {
            self.verify_device_id(firmware, &info.device.uid)?;
        }

        Ok(())
    }

    fn verify_device_id(&self, firmware: &[u8], device_uid: &[u8]) -> Result<()> {
        // Search for device UID in firmware
        if firmware.windows(device_uid.len()).any(|window| window == device_uid) {
            Ok(())
        } else {
            Err(Error::InvalidDeviceId)
        }
    }
}
