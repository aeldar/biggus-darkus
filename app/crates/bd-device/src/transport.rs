//! Line I/O under the device: the real serial port and the trait tests can fake.

use std::{
    io::{self, Read, Write},
    time::{Duration, Instant},
};

use serialport::SerialPort;

/// The board's console speed; 8N1 is the `serialport` default.
const BAUD: u32 = 9600;

/// The Nano resets when the port opens and needs this long before it reads lines.
/// <https://docs.arduino.cc/retired/hacking/software/AutoReset/>
const BOOT_WAIT: Duration = Duration::from_millis(2000);

/// Line-based link to the board.
pub trait Transport {
    /// Next line without its line break; `ErrorKind::TimedOut` if none arrives in time.
    fn read_line(&mut self, timeout: Duration) -> io::Result<String>;

    /// Send one line; the transport adds the line break.
    fn write_line(&mut self, line: &str) -> io::Result<()>;

    /// Drop unread input, so a late reply is not taken for the next command's reply.
    fn discard_input(&mut self) -> io::Result<()>;
}

/// A serial port at 9600 8N1.
pub struct SerialTransport {
    port: Box<dyn SerialPort>,
    /// Bytes read but not yet part of a full line.
    pending: Vec<u8>,
}

impl SerialTransport {
    /// Open `path` and wait for the board to boot.
    pub fn open(path: &str) -> io::Result<Self> {
        let port = serialport::new(path, BAUD).open()?;
        std::thread::sleep(BOOT_WAIT);
        Ok(Self {
            port,
            pending: Vec::new(),
        })
    }

    /// Cut the first full line off `pending`, if there is one.
    fn take_line(&mut self) -> Option<String> {
        let end = self.pending.iter().position(|&b| b == b'\n')?;
        let raw: Vec<u8> = self.pending.drain(..=end).collect();
        Some(
            String::from_utf8_lossy(&raw[..end])
                .trim_end_matches('\r')
                .to_owned(),
        )
    }
}

impl Transport for SerialTransport {
    fn read_line(&mut self, timeout: Duration) -> io::Result<String> {
        let deadline = Instant::now() + timeout;
        loop {
            if let Some(line) = self.take_line() {
                return Ok(line);
            }
            // One deadline for the whole line, not per read.
            let left = deadline
                .checked_duration_since(Instant::now())
                .filter(|d| !d.is_zero())
                .ok_or(io::ErrorKind::TimedOut)?;
            self.port.set_timeout(left)?;
            let mut buf = [0u8; 64];
            match self.port.read(&mut buf) {
                Ok(n) => self.pending.extend_from_slice(&buf[..n]),
                Err(e) if e.kind() == io::ErrorKind::TimedOut => {}
                Err(e) => return Err(e),
            }
        }
    }

    fn write_line(&mut self, line: &str) -> io::Result<()> {
        self.port.write_all(format!("{line}\n").as_bytes())?;
        self.port.flush()
    }

    fn discard_input(&mut self) -> io::Result<()> {
        self.pending.clear();
        Ok(self.port.clear(serialport::ClearBuffer::Input)?)
    }
}

/// A serial port found on this machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortInfo {
    /// Path to open, such as `/dev/ttyUSB0`.
    pub path: String,
    /// What kind of port it is, for the user to pick by.
    pub description: String,
}

/// All serial ports the system reports.
pub fn list_ports() -> io::Result<Vec<PortInfo>> {
    Ok(serialport::available_ports()?
        .into_iter()
        .map(|p| PortInfo {
            description: describe(&p.port_type),
            path: p.port_name,
        })
        .collect())
}

fn describe(kind: &serialport::SerialPortType) -> String {
    use serialport::SerialPortType::*;
    match kind {
        UsbPort(usb) => {
            let name = usb.product.as_deref().unwrap_or("USB serial");
            format!("{name} ({:04x}:{:04x})", usb.vid, usb.pid)
        }
        PciPort => "PCI".to_owned(),
        BluetoothPort => "Bluetooth".to_owned(),
        Unknown => "unknown".to_owned(),
    }
}
