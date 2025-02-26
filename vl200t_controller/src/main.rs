//! Blinks the LED on a Adafruit Feather RP2040 board
//!
//! This will blink on-board LED.
#![no_std]
#![no_main]

use core::fmt::Debug;

use adafruit_feather_rp2040::{
    hal::{
        clocks::{init_clocks_and_plls, Clock},
        gpio::FunctionSpi,
        pac,
        pio::PIOExt,
        spi::{Enabled, Spi, SpiDevice},
        usb::UsbBus,
        watchdog::Watchdog,
        Sio, Timer,
    },
    Pins, XOSC_CRYSTAL_FREQ,
};
use cortex_m::delay::Delay;
use cortex_m_rt::entry;
use embedded_hal::digital::v2::{InputPin, PinState};
use embedded_hal::{
    blocking::spi::{Transfer, Write as SpiWrite},
    digital::v2::OutputPin,
    spi::MODE_0,
};
use embedded_time::rate::*;
use panic_halt as _;
use smart_leds_trait::SmartLedsWrite;
use usb_device::{
    class_prelude::UsbBusAllocator,
    device::{UsbDeviceBuilder, UsbVidPid},
    UsbError,
};
use usbd_serial::USB_CLASS_CDC;
use ws2812_pio::Ws2812;

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    let mut watchdog = Watchdog::new(pac.WATCHDOG);

    let clocks = init_clocks_and_plls(
        XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let sio = Sio::new(pac.SIO);
    let pins = Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    let mut led_pin = pins.d13.into_push_pull_output();

    let mut chip_select_pin = pins.rx.into_push_pull_output();
    let aux_input_pin = pins.tx.into_pull_down_input();
    let _ = pins.miso.into_mode::<FunctionSpi>();
    let _ = pins.mosi.into_mode::<FunctionSpi>();
    let _ = pins.sclk.into_mode::<FunctionSpi>();
    let spi = Spi::<_, _, 8>::new(pac.SPI0).init(
        &mut pac.RESETS,
        125_000_000u32.Hz(),
        1_000_000u32.Hz(),
        &MODE_0,
    );
    chip_select_pin.set_high().unwrap();

    let usb_bus = UsbBusAllocator::new(UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));
    let mut serial = usbd_serial::SerialPort::new(&usb_bus);
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x1d50, 0x6173))
        .product("Serial port - Viltrox VL-200T Controller")
        .device_class(USB_CLASS_CDC)
        .build();

    // Configure the addressable LED
    let timer = Timer::new(pac.TIMER, &mut pac.RESETS);
    let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);
    let mut ws = Ws2812::new(
        // The onboard NeoPixel is attached to GPIO pin #16 on the Feather RP2040.
        pins.neopixel.into_mode(),
        &mut pio,
        sm0,
        clocks.peripheral_clock.freq(),
        timer.count_down(),
    );
    ws.write(core::iter::once((0, 0, 0))).unwrap();

    let mut transceiver = Transceiver::init_from(
        spi,
        chip_select_pin,
        aux_input_pin,
        cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().integer()),
    )
    .map_err(|e| {
        ws.write(core::iter::once((2, 0, 0))).unwrap();

        e
    })
    .unwrap();

    transceiver
        .verify_id()
        .map_err(|e| {
            ws.write(core::iter::once((0, 2, 0))).unwrap();
            e
        })
        .unwrap();

    ws.write(core::iter::once((1, 1, 1))).unwrap();

    let mut lights = [Light::default(); 6];
    let mut command_buffer = heapless::HistoryBuffer::<u8, { 3 * 6 + 1 }>::new();
    let mut first_command = true;
    let mut light_state = false;
    loop {
        let mut buf = [0u8; 64];

        if !usb_dev.poll(&mut [&mut serial]) {
            continue;
        }

        loop {
            match serial.read(&mut buf[..]) {
                Ok(count) => {
                    let input = &buf[..count];
                    for x in input {
                        command_buffer.write(*x);

                        if *x == ' ' as u8 {
                            let changed = process_command(&mut lights, &command_buffer.as_slice());
                            if changed || first_command {
                                first_command = false;

                                light_state = !light_state;
                                led_pin.set_state(PinState::from(light_state)).unwrap();

                                transceiver.set_lights(&lights);
                                transceiver.set_lights(&lights); // double up transmission, just in case

                                transceiver
                                    .verify_id()
                                    .map_err(|e| {
                                        ws.write(core::iter::once((0, 2, 0))).unwrap();
                                        e
                                    })
                                    .unwrap();
                            }
                        }
                    }
                }
                Err(UsbError::WouldBlock) => break, // No data received
                Err(err) => {
                    ws.write(core::iter::once((0, 0, 2))).unwrap();
                    break;
                } // An error occurred
            };
        }
    }
}

fn process_command(lights: &mut [Light; 6], commands: &[u8]) -> bool {
    if commands.len() != 3 * 6 + 1 {
        return false;
    }
    let position = commands.iter().position(|&x| x == ' ' as u8).unwrap();
    let (r, l) = commands.split_at(position);
    let mut command_buffer = [0u8; 3 * 6];
    l[1..]
        .iter()
        .chain(r.iter())
        .enumerate()
        .for_each(|(i, command_byte)| {
            command_buffer[i] = *command_byte;
        });

    let mut changed = false;
    command_buffer
        .chunks_exact(3)
        .enumerate()
        .for_each(|(light_idx, command_chunk)| {
            let power = command_chunk[0] == ('1' as u8);
            let brightness = command_chunk[1] - ('.' as u8) + 20;
            let temperature = command_chunk[2] - ('.' as u8) + 33;

            let light = &mut lights[light_idx];
            if light.on != power
                || light.brightness != brightness
                || light.temperature != temperature
            {
                changed = true;
                light.on = power;
                light.brightness = brightness;
                light.temperature = temperature;
            }
        });

    changed
}

#[derive(Debug, Clone, Copy)]
pub struct Light {
    pub on: bool,
    pub brightness: u8,
    pub temperature: u8,
}

impl Light {
    pub const MIN_BRIGHTNESS: u8 = 20;
    pub const MAX_BRIGHTNESS: u8 = 100;

    pub const MIN_TEMPERATURE: u8 = 33;
    pub const MAX_TEMPERATURE: u8 = 56;

    pub fn get_bound_brightness(&self) -> u8 {
        self.brightness
            .max(Self::MIN_BRIGHTNESS)
            .min(Self::MAX_BRIGHTNESS)
    }

    pub fn get_bound_temperature(&self) -> u8 {
        self.temperature
            .max(Self::MIN_TEMPERATURE)
            .min(Self::MAX_TEMPERATURE)
    }
}

impl Default for Light {
    fn default() -> Self {
        Self {
            on: false,
            brightness: Self::MIN_BRIGHTNESS,
            temperature: Self::MIN_TEMPERATURE,
        }
    }
}

#[derive(Debug)]
pub enum TransceiverInitError {
    IFFilterBankCalibrationError,
    VCOBankCalibrationError,
}

pub struct Transceiver<D: SpiDevice, I: InputPin, O: OutputPin>
where
    I::Error: Debug,
    O::Error: Debug,
{
    spi: Spi<Enabled, D, 8>,
    chip_select_pin: O,
    waiter_pin: I,
    delay: Delay,
}

impl<D: SpiDevice, I: InputPin, O: OutputPin> Transceiver<D, I, O>
where
    I::Error: Debug,
    O::Error: Debug,
{
    fn use_spi<R, F: FnOnce(&mut Spi<Enabled, D, 8>) -> R>(&mut self, f: F) -> R {
        self.chip_select_pin.set_low().unwrap();
        let r = f(&mut self.spi);
        self.delay.delay_us(1);
        self.chip_select_pin.set_high().unwrap();
        self.delay.delay_us(1);
        r
    }

    fn write(&mut self, buf: &[u8]) {
        self.use_spi(|spi| spi.write(buf).unwrap())
    }

    fn transfer_with(&mut self, buf: &mut [u8]) {
        self.use_spi(|spi| {
            spi.transfer(buf).unwrap();
        });
    }

    fn transfer<const DIMENSIONS: usize>(&mut self, mut buf: [u8; DIMENSIONS]) -> [u8; DIMENSIONS] {
        self.transfer_with(&mut buf);
        buf
    }

    fn run_init(&mut self) -> Result<(), TransceiverInitError> {
        // Reset transceiver by writing 0s to 'Mode' register
        self.write(&[0x00]);

        // Set up id register
        self.write(&[0x06, 0x57, 0x5a, 0x52, 0x46]);

        // Set up GPIO1 Pin to be MISO for SPI
        self.write(&[0x0b, 0b00011001]);

        // Set up GPIO2 Pin to be WTR
        self.write(&[0x0c, 0x01]);

        // Mode Control to 0x42 (Auto RSSI measurement while entering RX mode, AIF (Auto IF Offset), FIFO mode)
        self.write(&[0x01, 0x62]);

        // RC OSC Register 3
        //  BBCKS = 0, F_SYCK/8 as recommended
        self.write(&[0x09, 0x05]);

        // Clock register
        //  GRC = 0, F_XTAL x (DBL+1) / (GRC+1) = 2M when CGS == 1, (since it's 0, it's do-not-care)
        //  CSC = 1, system clock F_SYCK divider select, F_SYCK = F_MCLCK / 2
        //  CGS = 0, disables internal 32MHz PLL clock
        //  XS = 1, Crystal oscillator select is 1 meaning, crystal, not external clock
        self.write(&[0x0d, 0x05]);

        // Data rate = F_SYCK / 32 / {SDR + 1}.
        self.write(&[0x0e, 0x01]);

        //////// PLL
        // Pll 1
        // Channel = 4
        self.write(&[0x0f, 0x04]);

        // Pll 2
        // DBL = 1
        // RRC = 0
        // CHR = 15
        // BIP 8 = 0
        self.write(&[0x10, 0x9e]);

        // Pll 3
        // BIP = 0x4b = 75
        self.write(&[0x11, 0x4b]);

        // Pll 4
        // BFP 15 -> 8 = 0
        self.write(&[0x12, 0x00]);

        // Pll 5
        // BFP = 2
        self.write(&[0x13, 0x02]);
        // F_LO_BASE = F_PFD * (BIP + BFP/2^16) = (DBL + 1) * F_XTAL/(RRC+1) * (BIP + BFP/2^16)
        //                                      = 2         * 16Mhz /1       * (75  +   2/2^16)
        //                                      = 32 Mhz * (75 + 2^-15)
        //           = 2400.0009765625 ~= 2400.001 Mhz
        // F_CHSP = F_XTAL * (DBL+1) / 4 / (CHR+1)
        //        = 16Mhz  * 2       / 4 / (15 + 1)
        //        = 8Mhz / 16
        //        = 500 Khz

        // Tx 1 and 2
        self.write(&[0x14, 0x16]);
        self.write(&[0x15, 0x2b]);

        // Delay 1 and 2
        self.write(&[0x16, 0x12]);
        self.write(&[0x17, 0x40]);

        // Rx
        // RXSM = 3, FC = 0, RXDI = 0, DMG = 0, BWS = 1, ULS = 0
        self.write(&[0x18, 0x62]);

        // Rx gain 1 thru 4
        self.write(&[0x19, 0x80]);
        self.write(&[0x1a, 0x80]);
        self.write(&[0x1b, 0x00]);
        self.write(&[0x1c, 0x0a]);

        // Rssi Threshold
        self.write(&[0x1d, 0x32]);

        // Adc
        self.write(&[0x1e, 0xc3]);

        // Code Register 1 - Id and Preamble to be 4 bytes, enables CRC, disables data whitening and forward error correction
        self.write(&[0x1f, 0x0f]);

        // Code Register 2
        self.write(&[0x20, 0x16]);

        // Code Register 3 - Sets Data Whitening Seed to be 0 since we're not using it
        self.write(&[0x21, 0x00]);

        // If calibration 1
        self.write(&[0x22, 0x00]);

        // Vco current calibration - - Sets for manual calibrated value with VCO current manual calibration at [011]
        self.write(&[0x24, 0x13]);

        // Vco single band calibration 1 - Sets calibration value to be the auto value (not yet calibrated though)
        self.write(&[0x25, 0x00]);

        // Vco single band calibration 2 - VTH = [111] is vco tuning voltage upper threshold (1.3v). VTL = [011] is lower threshold (0.4v)
        self.write(&[0x26, 0x3b]);

        // Battery detect - disables
        self.write(&[0x27, 0x00]);

        // TX Test register
        // TX Current Setting = 0
        // PAC is 3
        // TX Buffer setting is 7
        // Basically max power (1.3 dBm, current at 21.25 mA)
        self.write(&[0x28, 0x1f]);

        // Rx Dem test 1  - sets DC estimation mode to average and hold
        // DC level is average value hold about 8 bit data rate later if preamble detected
        self.write(&[0x29, 0x47]);

        // Rx Dem test 2 - sets demodulator fix mode dc value to recommended 0x80 (but not used anyways since above, dc level is set to some average)
        self.write(&[0x2a, 0x80]);

        // CPC - sets charge pump current setting to recommended 2.0mA
        self.write(&[0x2b, 0x03]);

        // Crystal test - sets to required value
        self.write(&[0x2c, 0x01]);

        // PLL test - sets to required value
        self.write(&[0x2d, 0x45]);

        // VCO test 1 - sets to required value
        self.write(&[0x2e, 0x18]);

        // VCO test 2 - RF analog pin config for testing. sets to recommended
        self.write(&[0x2f, 0x00]);

        // IFAT - sets to required value
        self.write(&[0x30, 0x01]);

        // RSCale - sets to required value
        self.write(&[0x31, 0x0f]);

        // Filter test - sets to required value
        self.write(&[0x32, 0x00]);

        // Strobe command - Pll mode
        self.write(&[0xb0]);

        // Set calibration flags
        self.write(&[0x02, 0x03]);

        // Wait until calibration is done
        let mut buf: [u8; 2];
        loop {
            buf = [0x42u8, 0x00];
            self.transfer_with(&mut buf);
            if buf[1] == 0 {
                break;
            }
        }

        let if_filter_bank_calibration_success = self.transfer([0x62, 0x00])[1] & 0x10 == 0;
        if !if_filter_bank_calibration_success {
            return Err(TransceiverInitError::IFFilterBankCalibrationError);
        }
        let vco_bank_calibration_success = self.transfer([0x65, 0x00])[1] & 0x08 == 0;
        if !vco_bank_calibration_success {
            return Err(TransceiverInitError::VCOBankCalibrationError);
        }

        // Goto standby mode
        self.write(&[0xa0]);

        // Set up "Easy FIFO Mode"
        // Sets FIFO length to 16 bytes, 0s out PSA and FPM
        self.write(&[0x03, 0x0f]);
        self.write(&[0x04, 0x00]);

        return Ok(());
    }

    pub fn init_from(
        spi: Spi<Enabled, D, 8>,
        chip_select_pin: O,
        waiter_pin: I,
        delay: Delay,
    ) -> Result<Self, TransceiverInitError> {
        let mut transceiver = Self {
            spi,
            chip_select_pin,
            waiter_pin,
            delay,
        };

        transceiver.run_init()?;

        Ok(transceiver)
    }

    pub fn set_lights(&mut self, lights: &[Light; 6]) {
        // Fifo write pointer reset
        self.write(&[0xe0]);

        let power_state_byte = lights
            .iter()
            .enumerate()
            .map(|(idx, light)| if light.on { 1 << idx } else { 0 })
            .fold(0x40u8, core::ops::BitOr::bitor);

        // Set up message
        let message = [
            0x05,
            power_state_byte,
            lights[0].get_bound_brightness(),
            lights[0].get_bound_temperature(),
            lights[1].get_bound_brightness(),
            lights[1].get_bound_temperature(),
            lights[2].get_bound_brightness(),
            lights[2].get_bound_temperature(),
            lights[3].get_bound_brightness(),
            lights[3].get_bound_temperature(),
            lights[4].get_bound_brightness(),
            lights[4].get_bound_temperature(),
            lights[5].get_bound_brightness(),
            lights[5].get_bound_temperature(),
            Light::MAX_BRIGHTNESS, // Used for controlling all the lights at once, which we aren't doing
            Light::MAX_TEMPERATURE,
            0x00,
        ];
        self.write(&message);
        self.write(&[0xd0]);

        self.delay.delay_us(129);
        self.delay.delay_us(130 - 1 + 208 * 4 - 100);
        while self.waiter_pin.is_high().unwrap() {}

        self.write(&[0xa0]);
    }

    pub fn verify_id(&mut self) -> Result<(), ()> {
        let id_out = &self.transfer([0x46, 0x00, 0x00, 0x00, 0x00])[1..5];
        if id_out == [0x57, 0x5a, 0x52, 0x46] {
            Ok(())
        } else {
            Err(())
        }
    }
}
