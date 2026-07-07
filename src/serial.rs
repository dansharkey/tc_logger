use serialport::SerialPort;
use std::io::{Read, Write};
use std::time::Duration;

const CMD_START: &[u8] = &[0xAA, 0x55, 0x01, 0x03, 0x03];

pub fn list_ports() -> Vec<String> {
    serialport::available_ports()
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.port_name)
        .collect()
}

pub fn open(port_name: &str) -> Result<Box<dyn SerialPort>, serialport::Error> {
    serialport::new(port_name, 9600)
        .data_bits(serialport::DataBits::Eight)
        .stop_bits(serialport::StopBits::One)
        .parity(serialport::Parity::None)
        .timeout(Duration::from_millis(500))
        .open()
}

/// Request one real-time frame from the TA612C.
/// Response: 55 AA 01 0B [T1lo T1hi T2lo T2hi T3lo T3hi T4lo T4hi] [sum] = 13 bytes
/// Values are signed int16 little-endian, divide by 10 for °C.
/// Open circuit (0x7FFF) returned as NaN.
pub fn read_sample(port: &mut Box<dyn SerialPort>) -> Option<[f32; 4]> {
    port.write_all(CMD_START).ok()?;

    let mut buf = [0u8; 13];
    let mut n = 0;
    while n < 13 {
        match port.read(&mut buf[n..]) {
            Ok(k) if k > 0 => n += k,
            _ => return None,
        }
    }

    if buf[0] != 0x55 || buf[1] != 0xAA || buf[2] != 0x01 {
        return None;
    }

    let decode = |lo: u8, hi: u8| -> f32 {
        let raw = i16::from_le_bytes([lo, hi]);
        if raw == i16::MAX { f32::NAN } else { raw as f32 / 10.0 }
    };

    Some([
        decode(buf[4],  buf[5]),
        decode(buf[6],  buf[7]),
        decode(buf[8],  buf[9]),
        decode(buf[10], buf[11]),
    ])
}
