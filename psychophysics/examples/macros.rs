use std::time::{Duration, Instant};

macro_rules! for_frames {
    ($frame:ident, $var:ident in $iter:expr, $timeout:expr, $body:block) => {
        {
            let start = Instant::now();
            let timeout = Duration::from_secs($timeout);

            for $var in $iter {
                let $frame = $var;

                $body

                if start.elapsed() > timeout {
                    break;
                }
            }
        }
    };
}

fn main() {
    let numbers = [1, 2, 3, 4, 5];

    for_frames!(frame, num in numbers.iter(), 2, {
        println!("Number: {}", num);
        println!("{}", frame);
        std::thread::sleep(Duration::from_secs(1));
    });
}
