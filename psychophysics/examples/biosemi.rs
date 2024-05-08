use rustfft::{num_complex::Complex, FftPlanner};
use std::io::{self, Read};
use std::net::TcpStream;

fn main() -> io::Result<()> {
    const n_packets: usize = 10;
    const buffer_size: usize = 16 * n_packets;

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(buffer_size);

    let mut buffer = [0u8; buffer_size]; // 384 bytes buffer size
    let mut signal_buffer = vec![Complex { re: 0.0, im: 0.0 }; buffer_size];

    let mut stream = TcpStream::connect("localhost:778")?;

    println!("Connected to server");

    for _ in 0..100000 {
        println!("Parsing Segment consisting of {} packets", n_packets);
        let mut buffer_idx = 0;

        for _ in 0..n_packets {
            stream.read_exact(&mut buffer)?;
            for m in 0..16 {
                let offset = m * 24; // Each sample offset
                for ch in 0..8 {
                    let sample_pos = offset + ch * 3;
                    let sample = ((buffer[sample_pos + 2] as i32) << 16)
                        | ((buffer[sample_pos + 1] as i32) << 8)
                        | (buffer[sample_pos] as i32);
                    if ch == 0 {
                        // Only store channel 1
                        signal_buffer[buffer_idx] = Complex {
                            re: sample as f32,
                            im: 0.0,
                        };
                        buffer_idx += 1;
                    }
                }
            }
        }

        println!("Calculating DFT");
        fft.process(&mut signal_buffer);
        signal_buffer[0] = Complex { re: 0.0, im: 0.0 }; // Remove DC component

        println!("Segment processed");
    }

    Ok(())
}
