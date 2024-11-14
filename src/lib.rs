//! Device Firmware Update (DFU) Library
//! 
//! This library provides functionality for updating firmware on embedded devices
//! using either serial or network connections.
//! 
//! # Features
//! - Serial and TCP connection support
//! - Intel HEX firmware file parsing
//! - Automatic bootloader mode handling
//! - CRC-based verification
//! - Progress reporting
//! 
//! # Protocol Stack
//! - Application Protocol Layer (APL)
//! - Link Protocol Layer (LPL)
//! 
//! # Examples
//! 
//! ## Serial Device Update
//! ```rust
//! use fwupd::{DfuConfig, UpdateMode};
//! 
//! #[tokio::main]
//! async fn main() -> fwupd::Result<()> {
//!     // Direct connection to device
//!     let config = DfuConfig::new()
//!         .with_uri("serial:///dev/ttyUSB0")
//!         .with_firmware("firmware.hex")
//!         .with_update_mode(UpdateMode::Direct)
//!         .with_device_speed(9600)
//!         .with_update_speed(115200)
//!         .update()
//!         .verify();
//!
//!     let stream = tokio_serial::SerialStream::open(&config.uri)?;
//!     fwupd::update_firmware(stream, config).await
//! }
//! ```
//!
//! ## Network Device Update
//! ```rust
//! #[tokio::main]
//! async fn main() -> fwupd::Result<()> {
//!     let config = DfuConfig::new()
//!         .with_uri("tcp://192.168.1.100:5000")
//!         .with_firmware("firmware.hex")
//!         .update()
//!         .verify();
//!
//!     let stream = tokio::net::TcpStream::connect(&config.uri).await?;
//!     fwupd::update_firmware(stream, config).await
//! }
//! ```
//!
//! ## Reading Device Info
//! ```rust
//! #[tokio::main]
//! async fn main() -> fwupd::Result<()> {
//!     let config = DfuConfig::new()
//!         .with_uri("serial:///dev/ttyUSB0")
//!         .get_info();
//!
//!     let stream = tokio_serial::SerialStream::open(&config.uri)?;
//!     fwupd::read_device_info(stream).await
//! }
//! ```

mod dfu;
mod error;
mod protocols;

pub use dfu::{DfuStream, DfuConfig, UpdateMode, Command};
pub use error::{Error, Result};

use tokio::io::{AsyncRead, AsyncWrite};

/// Performs firmware update on a device
pub async fn update_firmware<T>(stream: T, config: DfuConfig) -> Result<()> 
where 
    T: AsyncRead + AsyncWrite + Unpin,
{
    let mut dfu = DfuStream::new(stream, config)?;
    dfu.update().await
}

/// Reads device information including bootloader version and device ID
pub async fn read_device_info<T>(stream: T) -> Result<()> 
where 
    T: AsyncRead + AsyncWrite + Unpin,
{
    let config = DfuConfig::new()
        .with_uri("stream")
        .get_info();

    let mut dfu = DfuStream::new(stream, config)?;
    dfu.update().await
}

/// Creates a new DFU configuration with default settings
pub fn new_config() -> DfuConfig {
    DfuConfig::new()
}
