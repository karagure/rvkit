use crate::config;
use std::io::{self, Read, Write};
use std::time::Duration;

pub fn run(port_arg: Option<&str>, baud_arg: Option<u32>) {
    let flash = config::try_load().map(|c| c.flash).unwrap_or_default();

    let port = port_arg
        .map(str::to_string)
        .or(flash.port)
        .unwrap_or_else(|| {
            eprintln!("Error: no serial port specified.");
            eprintln!("Pass --port <PORT> or set [flash] port in rvkit.toml.");
            list_available_ports();
            std::process::exit(1);
        });
    let baud = baud_arg.or(flash.baud_rate).unwrap_or(115200);

    println!(
        "Connecting to {} at {} baud... (Ctrl+C to quit)",
        port, baud
    );

    let mut serial = serialport::new(&port, baud)
        .timeout(Duration::from_millis(100))
        .open()
        .unwrap_or_else(|e| {
            eprintln!("Error: could not open {}: {}", port, e);
            list_available_ports();
            std::process::exit(1);
        });

    // Read raw bytes: serial output is not guaranteed to be UTF-8, and
    // line-buffering would drop partial lines on every read timeout.
    let mut stdout = io::stdout();
    let mut buf = [0u8; 1024];
    loop {
        match serial.read(&mut buf) {
            Ok(0) => {}
            Ok(n) => {
                stdout
                    .write_all(&buf[..n])
                    .expect("failed to write to stdout");
                stdout.flush().ok();
            }
            Err(e) if e.kind() == io::ErrorKind::TimedOut => {}
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => {
                eprintln!("\nRead error: {}", e);
                std::process::exit(1);
            }
        }
    }
}

fn list_available_ports() {
    match serialport::available_ports() {
        Ok(ports) if !ports.is_empty() => {
            eprintln!("Available ports:");
            for p in ports {
                eprintln!("  {}", p.port_name);
            }
        }
        _ => eprintln!("No serial ports detected."),
    }
}
