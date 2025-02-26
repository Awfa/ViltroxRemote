// Time [s],Packet ID,MOSI,MISO
// 3.625077400000000,0,0b  0000,0b  0000
use std::{fmt::Debug, str::FromStr};

use anyhow::Context;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Entry {
    time: f64,
    mosi: u8,
    miso: u8,
}

#[derive(Clone, PartialEq, Eq)]
enum Instruction {
    StrobeCommand(StrobeCommand),
    ReadControlRegister { register: Register, data: Vec<u8> },
    WriteControlRegister { register: Register, data: Vec<u8> },
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum Register {
    Mode = 0x00,
    ModeControl = 0x01,
    Calc = 0x02,
    Fifo1 = 0x03,
    Fifo2 = 0x04,
    FifoData = 0x05,
    IdData = 0x06,
    RcOsc1 = 0x07,
    RcOsc2 = 0x08,
    RcOsc3 = 0x09,
    CkoPin = 0x0A,
    Gpio1Pin1 = 0x0B,
    Gpio2Pin2 = 0x0C,
    Clock = 0x0D,
    DataRate = 0x0E,
    Pll1 = 0x0F,
    Pll2 = 0x10,
    Pll3 = 0x11,
    Pll4 = 0x12,
    Pll5 = 0x13,
    Tx1 = 0x14,
    Tx2 = 0x15,
    Delay1 = 0x16,
    Delay2 = 0x17,
    Rx = 0x18,
    RxGain1 = 0x19,
    RxGain2 = 0x1A,
    RxGain3 = 0x1B,
    RxGain4 = 0x1C,
    RssiThreshold = 0x1D,
    Adc = 0x1E,
    Code1 = 0x1F,
    Code2 = 0x20,
    Code3 = 0x21,
    IfCalibration1 = 0x22,
    IfCalibration2 = 0x23,
    VcoCurrentCalibration = 0x24,
    VcoSingleBandCalibration1 = 0x25,
    VcoSingleBandCalibration2 = 0x26,
    BatteryDetect = 0x27,
    TxTest = 0x28,
    RxDemTest1 = 0x29,
    RxDemTest2 = 0x2A,
    Cpc = 0x2B,
    CrystalTest = 0x2C,
    PllTest = 0x2D,
    VcoTest1 = 0x2E,
    VcoTest2 = 0x2F,
    IFat = 0x30,
    RScale = 0x31,
    FilterTest = 0x32,
}

impl Debug for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mode => write!(f, "Mode"),
            Self::ModeControl => write!(f, "ModeControl"),
            Self::Calc => write!(f, "Calc"),
            Self::Fifo1 => write!(f, "Fifo1"),
            Self::Fifo2 => write!(f, "Fifo2"),
            Self::FifoData => write!(f, "FifoData"),
            Self::IdData => write!(f, "IdData"),
            Self::RcOsc1 => write!(f, "RcOsc1"),
            Self::RcOsc2 => write!(f, "RcOsc2"),
            Self::RcOsc3 => write!(f, "RcOsc3"),
            Self::CkoPin => write!(f, "CkoPin"),
            Self::Gpio1Pin1 => write!(f, "Gpio1Pin1"),
            Self::Gpio2Pin2 => write!(f, "Gpio2Pin2"),
            Self::Clock => write!(f, "Clock"),
            Self::DataRate => write!(f, "DataRate"),
            Self::Pll1 => write!(f, "Pll1"),
            Self::Pll2 => write!(f, "Pll2"),
            Self::Pll3 => write!(f, "Pll3"),
            Self::Pll4 => write!(f, "Pll4"),
            Self::Pll5 => write!(f, "Pll5"),
            Self::Tx1 => write!(f, "Tx1"),
            Self::Tx2 => write!(f, "Tx2"),
            Self::Delay1 => write!(f, "Delay1"),
            Self::Delay2 => write!(f, "Delay2"),
            Self::Rx => write!(f, "Rx"),
            Self::RxGain1 => write!(f, "RxGain1"),
            Self::RxGain2 => write!(f, "RxGain2"),
            Self::RxGain3 => write!(f, "RxGain3"),
            Self::RxGain4 => write!(f, "RxGain4"),
            Self::RssiThreshold => write!(f, "RssiThreshold"),
            Self::Adc => write!(f, "Adc"),
            Self::Code1 => write!(f, "Code1"),
            Self::Code2 => write!(f, "Code2"),
            Self::Code3 => write!(f, "Code3"),
            Self::IfCalibration1 => write!(f, "IfCalibration1"),
            Self::IfCalibration2 => write!(f, "IfCalibration2"),
            Self::VcoCurrentCalibration => write!(f, "VcoCurrentCalibration"),
            Self::VcoSingleBandCalibration1 => write!(f, "VcoSingleBandCalibration1"),
            Self::VcoSingleBandCalibration2 => write!(f, "VcoSingleBandCalibration2"),
            Self::BatteryDetect => write!(f, "BatteryDetect"),
            Self::TxTest => write!(f, "TxTest"),
            Self::RxDemTest1 => write!(f, "RxDemTest1"),
            Self::RxDemTest2 => write!(f, "RxDemTest2"),
            Self::Cpc => write!(f, "Cpc"),
            Self::CrystalTest => write!(f, "CrystalTest"),
            Self::PllTest => write!(f, "PllTest"),
            Self::VcoTest1 => write!(f, "VcoTest1"),
            Self::VcoTest2 => write!(f, "VcoTest2"),
            Self::IFat => write!(f, "IFat"),
            Self::RScale => write!(f, "RScale"),
            Self::FilterTest => write!(f, "FilterTest"),
        }?;
        write!(f, " ({:#04x})", *self as u8)
    }
}

impl Debug for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StrobeCommand(arg0) => f.debug_tuple("StrobeCommand").field(arg0).finish(),
            Self::ReadControlRegister { register, data } => f
                .debug_struct("ReadControlRegister")
                .field("register", register)
                .field("data", &format_args!("0x{:02x?}", &data))
                .finish(),
            Self::WriteControlRegister { register, data } => f
                .debug_struct("WriteControlRegister")
                .field("register", register)
                .field("data", &format_args!("0x{:02x?}", &data))
                .finish(),
        }
    }
}

impl TryFrom<u8> for Register {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Register::Mode),
            0x01 => Ok(Register::ModeControl),
            0x02 => Ok(Register::Calc),
            0x03 => Ok(Register::Fifo1),
            0x04 => Ok(Register::Fifo2),
            0x05 => Ok(Register::FifoData),
            0x06 => Ok(Register::IdData),
            0x07 => Ok(Register::RcOsc1),
            0x08 => Ok(Register::RcOsc2),
            0x09 => Ok(Register::RcOsc3),
            0x0A => Ok(Register::CkoPin),
            0x0B => Ok(Register::Gpio1Pin1),
            0x0C => Ok(Register::Gpio2Pin2),
            0x0D => Ok(Register::Clock),
            0x0E => Ok(Register::DataRate),
            0x0F => Ok(Register::Pll1),
            0x10 => Ok(Register::Pll2),
            0x11 => Ok(Register::Pll3),
            0x12 => Ok(Register::Pll4),
            0x13 => Ok(Register::Pll5),
            0x14 => Ok(Register::Tx1),
            0x15 => Ok(Register::Tx2),
            0x16 => Ok(Register::Delay1),
            0x17 => Ok(Register::Delay2),
            0x18 => Ok(Register::Rx),
            0x19 => Ok(Register::RxGain1),
            0x1A => Ok(Register::RxGain2),
            0x1B => Ok(Register::RxGain3),
            0x1C => Ok(Register::RxGain4),
            0x1D => Ok(Register::RssiThreshold),
            0x1E => Ok(Register::Adc),
            0x1F => Ok(Register::Code1),
            0x20 => Ok(Register::Code2),
            0x21 => Ok(Register::Code3),
            0x22 => Ok(Register::IfCalibration1),
            0x23 => Ok(Register::IfCalibration2),
            0x24 => Ok(Register::VcoCurrentCalibration),
            0x25 => Ok(Register::VcoSingleBandCalibration1),
            0x26 => Ok(Register::VcoSingleBandCalibration2),
            0x27 => Ok(Register::BatteryDetect),
            0x28 => Ok(Register::TxTest),
            0x29 => Ok(Register::RxDemTest1),
            0x2A => Ok(Register::RxDemTest2),
            0x2B => Ok(Register::Cpc),
            0x2C => Ok(Register::CrystalTest),
            0x2D => Ok(Register::PllTest),
            0x2E => Ok(Register::VcoTest1),
            0x2F => Ok(Register::VcoTest2),
            0x30 => Ok(Register::IFat),
            0x31 => Ok(Register::RScale),
            0x32 => Ok(Register::FilterTest),
            _ => Err(anyhow::anyhow!("unable to decode register {}", value)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StrobeCommand {
    SleepMode,             // 1000
    IdleMode,              // 1001
    StandbyMode,           // 1010
    PllMode,               // 1011
    RxMode,                // 1100
    TxMode,                // 1101
    FifoWritePointerReset, // 1110
    FifoReadPointerReset,  // 1111
}

impl TryFrom<u8> for StrobeCommand {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0b1000 => Ok(StrobeCommand::SleepMode),
            0b1001 => Ok(StrobeCommand::IdleMode),
            0b1010 => Ok(StrobeCommand::StandbyMode),
            0b1011 => Ok(StrobeCommand::PllMode),
            0b1100 => Ok(StrobeCommand::RxMode),
            0b1101 => Ok(StrobeCommand::TxMode),
            0b1110 => Ok(StrobeCommand::FifoWritePointerReset),
            0b1111 => Ok(StrobeCommand::FifoReadPointerReset),
            _ => Err(anyhow::anyhow!("unable to decode strobe command {}", value)),
        }
    }
}

fn read_entries(entries: &[Entry]) -> anyhow::Result<(f64, Vec<Instruction>)> {
    let mut result = Vec::new();
    let mut i = 0;
    while i < entries.len() {
        let entry = &entries[i];
        let consumed_nibbles;
        let instruction = if entry.mosi >> 3 == 1 {
            consumed_nibbles = 2;
            Instruction::StrobeCommand(StrobeCommand::try_from(entry.mosi).context(format!(
                "on entry {} = {:?}. decoded instructions = {:?}",
                i, entry, result
            ))?)
        } else {
            let register = Register::try_from((entry.mosi << 4 | entries[i + 1].mosi) & 0b111111)
                .context(format!(
                "on entry {} = {:?}. decoded instructions = {:?}",
                i, entry, result
            ))?;

            let data_range = match register {
                Register::IdData | Register::FifoData => &entries[i + 2..],
                _ => &entries[i + 2..usize::min(i + 4, entries.len())],
            };
            let data: Vec<u8> = data_range
                .chunks_exact(2)
                .map(|chunk| chunk[0].mosi << 4 | chunk[1].mosi)
                .collect();

            consumed_nibbles = 2 + data.len() * 2;
            if entry.mosi >> 2 == 1 {
                Instruction::ReadControlRegister { register, data }
            } else {
                Instruction::WriteControlRegister { register, data }
            }
        };
        i += consumed_nibbles;
        result.push(instruction);
    }
    Ok((entries[0].time, result))
}

fn main() -> anyhow::Result<()> {
    let mut csv_reader = csv::Reader::from_reader(
        std::fs::OpenOptions::new()
            .read(true)
            .open("rust.csv")?,
    );

    let mut records = Vec::new();
    {
        let mut record_frame = Vec::new();
        for result in csv_reader.records() {
            let record = result?;
            let type_str = record.get(1).unwrap();
            let time_str = record.get(2).unwrap();
            let mosi_str = record.get(4).unwrap();
            match type_str {
                "disable" => {
                    if !record_frame.is_empty() {
                        records.push(record_frame);
                        record_frame = Vec::new();
                    }
                }
                "result" => {
                    let time = f64::from_str(time_str)?;
                    let mosi = u8::from_str_radix(&mosi_str[2..], 2)?;
                    record_frame.push(Entry {
                        time,
                        mosi,
                        miso: 0,
                    });
                }
                "enable" => {}
                _ => unimplemented!(),
            }
        }
    }
    let instructions = records
        .iter()
        .map(|frame| read_entries(&frame).unwrap())
        .collect::<Vec<_>>();
    let clumped_instructions = instructions.iter().fold(
        Vec::new(),
        |mut acc: Vec<Vec<(f64, &[Instruction])>>, e: &(f64, Vec<Instruction>)| {
            match acc.last_mut() {
                Some(last) => {
                    if e.0 - last.last().unwrap().0 < 0.2 {
                        last.push((e.0, &e.1))
                    } else {
                        acc.push(vec![(e.0, &e.1)]);
                    }
                }
                None => {
                    acc.push(vec![(e.0, &e.1)]);
                }
            };
            acc
        },
    );

    let x = false;
    let expected_groupings: &[&str] = if x {
        &[
            "Remote on",
            "Power off",
            "Power on",
            "Brightness up 1",
            "Brightness up 2",
            "Brightness up 3",
            "Brightness down 1",
            "Brightness down 2",
            "Brightness down 3",
            "Temperature up 1",
            "Temperature up 2",
            "Temperature up 3",
            "Temperature down 1",
            "Temperature down 2",
            "Temperature down 3",
            "Group toggle",
        ]
    } else {
        &[
            "Power on - Group A",
            // "Power off - Group A",
            // "Group change - Group A to B",
            // "Power on - Group B",
            // "Power off - Group B",
            // "Group change - Group B to C",
            // "Power on - Group C",
            // "Power off - Group C",
            // "Group change - Group C to D",
            // "Power on/off - Group D",
            // "Power on/off - Group D",
            // "Group change - Group D to E",
            // "Power on/off - Group E",
            // "Power on/off - Group E",
            // "Group change - Group E to F",
            // "Power on/off - Group F",
            // "Power on/off - Group F",
        ]
    };

    let mut covered_groupings = 0;
    for (instructions, grouping) in clumped_instructions.iter().zip(
        expected_groupings
            .into_iter()
            .chain(std::iter::repeat(&"Unknown group")),
    ) {
        covered_groupings += 1;
        println!("{} = [", grouping);
        for instruction in instructions {
            println!("  {:?}", instruction.1);
        }
        println!("]");
        println!();
    }

    if covered_groupings < expected_groupings.len() {
        println!(
            "Unmatched groupings: {:?}",
            &expected_groupings[covered_groupings..]
        );
    }

    Ok(())
}
