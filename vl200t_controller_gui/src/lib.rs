use serialport::SerialPort;

pub mod app;

pub struct LightSettings {
    pub on: [bool; 6],
    pub brightness: [u8; 6],
    pub temperature: u8,
}

pub struct Device {
    port: Box<dyn SerialPort>,
}
