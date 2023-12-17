use rand::Rng;
use serialport::{available_ports, SerialPort};
use spin_sleep;
use std::io::BufReader;
use std::io::{self, Read, Write};
use std::str;
use std::time::{Duration, Instant};

fn main() -> io::Result<()> {
    // List available ports and select the first one (modify as needed)
    let ports = available_ports().expect("No ports found!");
    if ports.is_empty() {
        println!("No serial ports found.");
        return Ok(());
    }
    let port_name = &ports[2].port_name;
    println!("Using port: {}", port_name);

    // Open the first serial port available
    let mut port = serialport::new(port_name, 115200)
        .timeout(Duration::from_millis(2000))
        .open()?;

    let spin_sleeper = spin_sleep::SpinSleeper::new(10000_000)
        .with_spin_strategy(spin_sleep::SpinStrategy::YieldThread);

    let mut last_time = Instant::now();

    loop {
        // print delay between messages
        let start = Instant::now();
        println!("Delay: {:?}", start - last_time);
        last_time = start;
        // Message to send (generate one random byte using the rand crate)
        let message_byte = rand::thread_rng().gen::<u8>();

        // Send the message
        port.write_all(&[message_byte])?;

        // Buffer to store the incoming data
        let mut buffer = [0; 32];

        // Read the response
        match port.read(&mut buffer) {
            Ok(_) => {
                // Stop the timer and calculate latency
                let duration = start.elapsed();

                // check if the response is identical to the sent message
                if buffer[0] != message_byte {
                    println!("Received incorrect response!");
                }

                println!(
                    "Received response {}. Roundtrip time: {:?}",
                    buffer[0], duration
                );
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                println!("Timed out waiting for response.");
            }
            Err(e) => return Err(e),
        }

        // wait for 10ms (but take into account the time it took to receive the response)
        spin_sleeper.sleep(Duration::from_millis(10) - start.elapsed());
    }
}
