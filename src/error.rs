use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Invalid packet type: {0}")]
    InvalidPacket(u8),

    #[error("CRC mismatch")]
    CrcMismatch,

    #[error("Timeout")]
    Timeout,

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("No firmware file specified")]
    NoFirmwareFile,

    #[error("Hex file error: {0}")]
    HexFileError(#[from] ihex::Error),

    #[error("Firmware too large for device")]
    FirmwareTooLarge,

    #[error("Invalid device ID")]
    InvalidDeviceId,

    #[error("Verification failed")]
    VerificationFailed,

    #[error("Bootloader not detected")]
    BootloaderNotDetected,

    #[error("Invalid configuration: {0}")]
    Configuration(String),
}

pub type Result<T> = std::result::Result<T, Error>;
