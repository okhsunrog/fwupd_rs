use super::types::{DfuConfig, UpdateMode};

impl Default for DfuConfig {
    fn default() -> Self {
        Self {
            uri: String::new(),
            filename: None,
            block_size: 1024,
            get_info: false,
            update: false,
            overwrite: false,
            verify: false,
            quit: false,
            dev_netid: 0,
            dev_speed: 9600,
            upd_speed: 115200,
            lnk_speed: 9600,
            upd_mode: UpdateMode::None,
            gap_filling: 0xFF,
        }
    }
}

impl DfuConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_uri(mut self, uri: impl Into<String>) -> Self {
        self.uri = uri.into();
        self
    }

    pub fn with_firmware(mut self, filename: impl Into<String>) -> Self {
        self.filename = Some(filename.into());
        self
    }

    pub fn with_block_size(mut self, size: usize) -> Self {
        self.block_size = size;
        self
    }

    pub fn with_update_mode(mut self, mode: UpdateMode) -> Self {
        self.upd_mode = mode;
        self
    }

    pub fn with_device_speed(mut self, speed: usize) -> Self {
        self.dev_speed = speed;
        self
    }

    pub fn with_update_speed(mut self, speed: usize) -> Self {
        self.upd_speed = speed;
        self
    }

    pub fn with_link_speed(mut self, speed: usize) -> Self {
        self.lnk_speed = speed;
        self
    }

    pub fn get_info(mut self) -> Self {
        self.get_info = true;
        self
    }

    pub fn update(mut self) -> Self {
        self.update = true;
        self
    }

    pub fn verify(mut self) -> Self {
        self.verify = true;
        self
    }

    pub fn overwrite(mut self) -> Self {
        self.overwrite = true;
        self
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.uri.is_empty() {
            return Err("URI must be specified");
        }

        if self.update && self.filename.is_none() {
            return Err("Firmware file must be specified for update");
        }

        Ok(())
    }
}
