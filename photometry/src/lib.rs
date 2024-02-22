use nalgebra::Matrix3;
use serialport::SerialPort;

pub struct ColorCalII {
    serial: Box<dyn SerialPort>,
    calibration_matrix: Matrix3<f32>,
    serial_num: u32,
    firmware: u32,
    firm_build: u32,
}

impl std::fmt::Debug for ColorCalII {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CRS ColorCalII Photometer")
            .field("Serial Number: ", &self.serial_num)
            .field("Firmware: ", &self.firmware)
            .field("Firmware Build: ", &self.firm_build)
            .finish()
    }
}

impl ColorCalII {
    pub fn new(port: &str) -> Result<Self, serialport::Error> {
        let mut serial: Box<dyn SerialPort> =
            serialport::new("/dev/tty.usbmodem00001", 115200)
                .timeout(std::time::Duration::from_millis(10000))
                .open()
                .expect("Failed to open port");

        let (serial_num, firmware, firm_build) = Self::get_device_info(&mut *serial)?;

        let calib_matrix = Self::get_color_matrix(&mut *serial, 0)?;
        println!("{:?}", calib_matrix);

        Ok(Self {
            serial,
            calibration_matrix: calib_matrix,
            serial_num,
            firmware,
            firm_build,
        })
    }

    /// Sends a message to the device and returns the response
    fn send_msg_response(
        serial: &mut dyn SerialPort,
        msg: String,
    ) -> Result<String, serialport::Error> {
        serial.write(msg.as_bytes())?;
        // send a newline to execute the command
        serial.write("\n".as_bytes())?;
        // flush the buffer
        serial.flush()?;

        // listen for the response
        // we wait until we receive "/r/n>" to know the response is done
        let mut buffer: Vec<u8> = vec![0; 1];
        let mut response = String::new();
        loop {
            serial.read(&mut buffer)?;
            response.push(buffer[0] as char);
            if response.ends_with("\n\r>") {
                break;
            }
        }

        // return the response
        Ok(response)
    }

    fn needs_calibration(&mut self) -> bool {
        // check if the calibration matrix is the identity matrix
        self.calibration_matrix != Matrix3::identity()
    }

    /// Sends a message to the device without waiting for a response
    fn send_msg(
        serial: &mut dyn SerialPort,
        msg: String,
    ) -> Result<(), serialport::Error> {
        serial.write(msg.as_bytes())?;
        Ok(())
    }

    fn get_device_info(
        serial: &mut dyn SerialPort,
    ) -> Result<(u32, u32, u32), serialport::Error> {
        let val = Self::send_msg_response(serial, "IDR".to_string())?;
        let valstrip = val.trim_end_matches("\n\r>");
        let val = valstrip.split(',').collect::<Vec<&str>>();
        let ok = val[0] == "OK00";
        let firmware = val[2];
        let serial_num = val[4];
        let firm_build = val.last().unwrap();

        // trim and parse to u32
        let serial_num = serial_num.trim().parse::<u32>().unwrap();
        let firmware = firmware.trim().parse::<u32>().unwrap();
        let firm_build = firm_build.trim().parse::<u32>().unwrap();

        Ok((serial_num, firmware, firm_build))
    }

    /// Obtains one of the calibration matrices from the device. ColorCalII has 3 matrices, with
    /// only the first one usually used.
    fn get_color_matrix(
        serial: &mut dyn SerialPort,
        matrix_id: u32,
    ) -> Result<Matrix3<f32>, serialport::Error> {
        // check if the matrix_id is valid
        if matrix_id > 2 {
            return Err(serialport::Error::new(
                serialport::ErrorKind::InvalidInput,
                "Invalid matrix_id",
            ));
        }

        let matrix_id = matrix_id as usize;

        let mut matrix = Matrix3::zeros();
        for row_n in 0..3 {
            let row_name = format!("r0{}", row_n + 1 + matrix_id * 3);
            let val = Self::send_msg_response(serial, row_name)?;
            let valstrip = val.trim_start_matches("\n\r");
            let valstrip = valstrip.trim_end_matches("\n\r>");
            // split by comma and trim whitespace
            let vals = valstrip.split(",").map(|x| x.trim()).collect::<Vec<&str>>();
            if vals[0] == "OK00" && vals.len() > 1 {
                // trim and parse to f32

                let raw_vals = vals[1..].iter().map(|x| x.parse::<u32>().unwrap());
                let floats = raw_vals.map(Self::minolta2float);
                matrix
                    .row_mut(row_n)
                    .copy_from(&nalgebra::RowVector3::from_iterator(floats));
            } else {
                panic!("Error reading calibration matrix. Response: {}", val);
            }
        }
        Ok(matrix)
    }

    pub fn measure(&mut self) -> Result<(f32, f32, f32), serialport::Error> {
        let val = Self::send_msg_response(&mut *self.serial, "MES".to_string())?;
        let valstrip = val.trim_end_matches("\n\r>");
        let valstrip = valstrip.trim_start_matches("\n\r");

        let vals = valstrip.split(',').collect::<Vec<&str>>();
        println!("{:?}", vals);
        let ok = vals[0] == "OK00";
        if !ok {
            panic!("Error reading measurement. Response: {}", val);
        }
        let xyz_raw = vals[1..]
            .iter()
            .map(|x| x.trim().parse::<f32>().unwrap())
            .collect::<Vec<f32>>();
        println!("{:?}", xyz_raw);
        let xyz: nalgebra::Vector3<f32> =
            nalgebra::Vector3::new(xyz_raw[0], xyz_raw[1], xyz_raw[2]);
        let xyz = self.calibration_matrix * xyz;
        Ok((xyz[0], xyz[1], xyz[2]))
    }

    /// Convert an integer in the Minolta format to a float
    pub fn minolta2float(in_val: u32) -> f32 {
        let in_val = in_val as f32;

        if in_val < 50000.0 {
            in_val / 10000.0
        } else {
            (-in_val + 50000.0) / 10000.0
        }
    }
}

// use nalgebra;

// /// A spectrum is a set of wavelenghts and their corresponding streng
// pub trait Spectrum<T, const N: usize> {
//     fn wavelengths(&self) -> [T; N];
//     fn strengths(&self) -> [T; N];
// }

// /// A set of three color matching functions
// pub trait CMF<T, const N: usize> {
//     fn x(&self) -> [T; N];
//     fn y(&self) -> [T; N];
//     fn z(&self) -> [T; N];
//     fn wavelengths(&self) -> [T; N];
// }

// /// The CIE 1931 color matching functions for a 2 degree standard observer
// pub struct CIE1931 {}
// impl CMF<f32, 95> for CIE1931 {
//     fn wavelengths(&self) -> [f32; 95] {
//         [
//             360.0, 365.0, 370.0, 375.0, 380.0, 385.0, 390.0, 395.0, 400.0, 405.0, 410.0,
//             415.0, 420.0, 425.0, 430.0, 435.0, 440.0, 445.0, 450.0, 455.0, 460.0, 465.0,
//             470.0, 475.0, 480.0, 485.0, 490.0, 495.0, 500.0, 505.0, 510.0, 515.0, 520.0,
//             525.0, 530.0, 535.0, 540.0, 545.0, 550.0, 555.0, 560.0, 565.0, 570.0, 575.0,
//             580.0, 585.0, 590.0, 595.0, 600.0, 605.0, 610.0, 615.0, 620.0, 625.0, 630.0,
//             635.0, 640.0, 645.0, 650.0, 655.0, 660.0, 665.0, 670.0, 675.0, 680.0, 685.0,
//             690.0, 695.0, 700.0, 705.0, 710.0, 715.0, 720.0, 725.0, 730.0, 735.0, 740.0,
//             745.0, 750.0, 755.0, 760.0, 765.0, 770.0, 775.0, 780.0, 785.0, 790.0, 795.0,
//             800.0, 805.0, 810.0, 815.0, 820.0, 825.0, 830.0,
//         ]
//     }
//     fn x(&self) -> [f32; 95] {
//         [
//             0.000129900000,
//             0.000232100000,
//             0.000414900000,
//             0.000741600000,
//             0.001368000000,
//             0.002236000000,
//             0.004243000000,
//             0.007650000000,
//             0.014310000000,
//             0.023190000000,
//             0.043510000000,
//             0.077630000000,
//             0.134380000000,
//             0.214770000000,
//             0.283900000000,
//             0.328500000000,
//             0.348280000000,
//             0.348060000000,
//             0.336200000000,
//             0.318700000000,
//             0.290800000000,
//             0.251100000000,
//             0.195360000000,
//             0.142100000000,
//             0.095640000000,
//             0.057950010000,
//             0.032010000000,
//             0.014700000000,
//             0.004900000000,
//             0.002400000000,
//             0.009300000000,
//             0.029100000000,
//             0.063270000000,
//             0.109600000000,
//             0.165500000000,
//             0.225749900000,
//             0.290400000000,
//             0.359700000000,
//             0.433449900000,
//             0.512050100000,
//             0.594500000000,
//             0.678400000000,
//             0.762100000000,
//             0.842500000000,
//             0.916300000000,
//             0.978600000000,
//             1.026300000000,
//             1.056700000000,
//             1.062200000000,
//             1.045600000000,
//             1.002600000000,
//             0.938400000000,
//             0.854449900000,
//             0.751400000000,
//             0.642400000000,
//             0.541900000000,
//             0.447900000000,
//             0.360800000000,
//             0.283500000000,
//             0.218700000000,
//             0.164900000000,
//             0.121200000000,
//             0.087400000000,
//             0.063600000000,
//             0.046770000000,
//             0.032900000000,
//             0.022700000000,
//             0.015840000000,
//             0.011359160000,
//             0.008110916000,
//             0.005790346000,
//             0.004109457000,
//             0.002899327000,
//             0.002049190000,
//             0.001439971000,
//             0.000999949300,
//             0.000690078600,
//             0.000476021300,
//             0.000332301100,
//             0.000234826100,
//             0.000166150500,
//             0.000117413000,
//             0.000083075270,
//             0.000058706520,
//             0.000041509940,
//             0.000029353260,
//             0.000020673830,
//             0.000014559770,
//             0.000010253980,
//             0.000007221456,
//             0.000005085868,
//             0.000003581652,
//             0.000002522525,
//             0.000001776509,
//             0.000001251141,
//         ]
//     }

//     fn y(&self) -> [f32; 95] {
//         todo!()
//     }

//     fn z(&self) -> [f32; 95] {
//         todo!()
//     }
// }
// /// The CIE 1931 color matching functions for the Y value
// pub struct CIE1931Y {}
// /// The CIE 1931 color matching functions for the Z value
// pub struct CIE1931Z {}

// pub trait XYZColorSpace<T, C1, C2, C3> {
//     fn from_spectrum(spectrum: &dyn Spectrum<T, 3>) -> Self;
// }

// /// A color defined by its XYZ coordinates given three color matching functions
// pub struct XYZ<T, C, const N: usize>
// where
//     T: nalgebra::Scalar,
//     C: CMF<T, N>,
// {
//     pub x: T,
//     pub y: T,
//     pub z: T,

//     /// The color matching functions to define XYZ
//     _cmf: C,
// }

// pub struct RGB<T, P>
// where
//     T: nalgebra::Scalar,
//     P: RGBPrimaries<T>,
// {
//     /// The red value
//     r: T,
//     /// The green value
//     g: T,
//     /// The blue value
//     b: T,
//     ///
//     /// The reference XYZ space
//     _refernce_space: XYZ<T, CIE1931X, CIE1931Y, CIE1931Z>,
//     /// Red primary in XY coordinates
//     _primaries: P,

//     _
// }

// pub trait RGBPrimaries<T> {
//     fn red(&self) -> (T, T);
//     fn green(&self) -> (T, T);
//     fn blue(&self) -> (T, T);
// }
