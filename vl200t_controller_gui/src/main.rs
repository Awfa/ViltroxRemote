use anyhow::{Result, Context};
use serialport::{SerialPortInfo, SerialPortType};

fn main() -> Result<()> {
    let port = serialport::available_ports()
        .unwrap()
        .into_iter()
        .find(|serial| match serial.port_type {
            SerialPortType::UsbPort(ref usb_info) => {
                usb_info.vid == 0x1d50 && usb_info.pid == 0x6173
            }
            SerialPortType::PciPort | SerialPortType::BluetoothPort | SerialPortType::Unknown => {
                false
            }
        });
    let port = port.ok_or_else(|| anyhow::anyhow!("Couldn't find controller device"))?;
    run(&port)
}

fn run(port: &SerialPortInfo) -> Result<()> {
    let app = vl200t_controller_gui::app::ControllerApp::new(
        serialport::new(&port.port_name, 9600)
            .data_bits(serialport::DataBits::Eight)
            .open().with_context(|| format!("Opening serial port {}", port.port_name))?,
    );
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
