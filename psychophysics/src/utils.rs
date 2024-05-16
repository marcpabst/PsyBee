use std::fmt::Display;


use bytemuck;


pub(crate) trait AtomicExt<T> {
    fn load_relaxed(&self) -> T;
    fn store_relaxed(&self, value: T);
}

impl AtomicExt<f64> for atomic_float::AtomicF64 {
    fn load_relaxed(&self) -> f64 {
        self.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn store_relaxed(&self, value: f64) {
        self.store(value, std::sync::atomic::Ordering::Relaxed);
    }
}

impl AtomicExt<f32> for atomic_float::AtomicF32 {
    fn load_relaxed(&self) -> f32 {
        self.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn store_relaxed(&self, value: f32) {
        self.store(value, std::sync::atomic::Ordering::Relaxed);
    }
}

impl AtomicExt<usize> for std::sync::atomic::AtomicUsize {
    fn load_relaxed(&self) -> usize {
        self.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn store_relaxed(&self, value: usize) {
        self.store(value, std::sync::atomic::Ordering::Relaxed);
    }
}

impl AtomicExt<u32> for std::sync::atomic::AtomicU32 {
    fn load_relaxed(&self) -> u32 {
        self.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn store_relaxed(&self, value: u32) {
        self.store(value, std::sync::atomic::Ordering::Relaxed);
    }
}

impl AtomicExt<bool> for std::sync::atomic::AtomicBool {
    fn load_relaxed(&self) -> bool {
        self.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn store_relaxed(&self, value: bool) {
        self.store(value, std::sync::atomic::Ordering::Relaxed);
    }
}


pub use web_time as time;

use crate::errors::{self, PsychophysicsError};

/// Nonblocking logging. When using the `pyo3-log` crate, logging from a background thread can under certain circumstances cause a deadlock.
/// This function is a workaround for this issue. It logs the message in a separate thread, but doesn't wait for the logging to finish.
/// This is very bad for performance, but it's better than a deadlock.
/// Example:
/// ```
/// log_nonblocking!(log::Level::Info, "This is a nonblocking log message");
/// ```
#[macro_export]
macro_rules! log_nonblocking {
    ($lvl:expr, $($arg:tt)*) => {
        if log::log_enabled!($lvl) {
            // then we log the message in a separate thread
            std::thread::spawn(move || {
                log::log!($lvl, $($arg)*);
            });
        }
    };
}



/// Includes an image as a reference to a byte array.
/// This is useful for including images in the binary.
/// The image is loaded at compile time.
///
/// # Example
///
/// ```
/// let image = include_image!("image.png");
/// ```
#[macro_export]
macro_rules! include_image {
    ($name:literal) => {{
        let bytes = include_bytes!($name);
        image::load_from_memory(bytes).unwrap()
    }};
}

#[macro_export]
macro_rules! impl_into_string_vector_tuple {
    () => (
        impl IntoStringVector for () {
            fn into_string_vec(self) -> Vec<String> {
                vec![]
            }
        }
    );

    ( $($name:ident)+) => (
        impl<$($name: Display),*> IntoStringVector for ($($name,)*) {

            fn into_string_vec(self) -> Vec<String> {
                let ($($name,)*) = self;
                vec![$($name.to_string()),*]
            }
        }


    );
}

macro_rules! for_each_tuple_ {
    ( $m:ident !! ) => (
    $m! { }
    );
    ( $m:ident !! $h:ident, $($t:ident,)* ) => (
    $m! { $h $($t)* }
    for_each_tuple_! { $m !! $($t,)* }
    );
   }
macro_rules! for_each_tuple {
    // implentation for 1 to 40 elements
    ( $m:ident ) => (
        for_each_tuple_! { $m !! T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11,T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,T22, T23, T24, T25, T26, T27, T28, T29, T30, T31,T32, T33, T34, T35, T36, T37, T38, T39, T40, }
    );
   }
// Generate implementations for tuples
for_each_tuple!(impl_into_string_vector_tuple);

pub trait IntoStringVector {
    fn into_string_vec(self) -> Vec<String>;
}

impl IntoStringVector for Vec<String> {
    fn into_string_vec(self) -> Vec<String> {
        self
    }
}

fn check_unique_column_names<I, S>(column_names: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: Into<String> + 'static,
{
    let column_names: Vec<String> = column_names.into_iter().map(Into::into).collect();

    // check that all column names are unique
    let mut unique_column_names = column_names.clone();
    unique_column_names.sort();
    unique_column_names.dedup();

    unique_column_names.len() == column_names.len()
}

/// Event manager that handles "events" that occur during the experiment.
/// Depending on the backend, this can be written to a file, sent over the network, etc.
pub struct CSVEventLogger {
    filepath: std::path::PathBuf,
    columns: Vec<String>,
    delimiter: u8,
    writer: csv::Writer<std::fs::File>,
}

impl CSVEventLogger {
    /// Create a new CSVEventLogger.
    pub fn new<I, S, P>(
        path: P,
        columns: I,
        delimiter: u8,
        overwrite: bool,
    ) -> Result<Self, errors::PsychophysicsError>
    where
        P: Into<std::path::PathBuf>,
        I: IntoIterator<Item = S>,
        S: Into<String> + 'static,
    {
        let filepath = path.into();

        let columns: Vec<String> = columns.into_iter().map(Into::into).collect();

        if !check_unique_column_names(columns.clone()) {
            return Err(errors::PsychophysicsError::ColumnNamesNotUniqueError);
        }

        // check if file exists
        if filepath.exists() {
            // check if file is empty
            if std::fs::metadata(&filepath).unwrap().len() > 0 {
                // if overwrite is true, delete the file, otherwise return an error
                if overwrite {
                    std::fs::remove_file(&filepath)?;
                } else {
                    return Err(errors::PsychophysicsError::FileExistsAndNotEmptyError(
                        filepath.to_string_lossy().to_string(),
                    ));
                }
            }
        }

        let mut writer = csv::WriterBuilder::new()
            .has_headers(false)
            .delimiter(delimiter)
            .from_path(&filepath)?;

        writer.write_record(columns.clone())?;

        Ok(Self {
            filepath,
            columns,
            delimiter,
            writer,
        })
    }

    /// Log an event.
    pub fn log<I>(&mut self, column_values: I) -> Result<(), PsychophysicsError>
    where
        I: IntoStringVector,
    {
        // convert to vector
        let event: Vec<String> = column_values.into_string_vec();

        // make sure the event has the correct number of columns
        if event.len() != self.columns.len() {
            return Err(errors::PsychophysicsError::DataLengthMismatchError(
                event.len(),
                self.columns.len(),
            ));
        }

        // write event to file
        self.writer.write_record(&event)?;
        self.writer.flush()?;

        Ok(())
    }

    pub fn log_cols<I, J>(
        &mut self,
        column_names: I,
        column_values: J,
    ) -> Result<(), PsychophysicsError>
    where
        I: IntoStringVector,
        J: IntoStringVector,
    {
        let column_names: Vec<String> = column_names.into_string_vec();
        let column_values: Vec<String> = column_values.into_string_vec();

        // check that all column names are unique
        if !check_unique_column_names(column_names.clone()) {
            return Err(errors::PsychophysicsError::ColumnNamesNotUniqueError);
        }

        // assert that all column names are in self.columns
        for column_name in column_names.iter() {
            if !self.columns.contains(column_name) {
                return Err(errors::PsychophysicsError::ColumnNameDoesNotExistError(
                    column_name.clone(),
                ));
            }
        }

        if column_names.len() != column_values.len() {
            return Err(errors::PsychophysicsError::DataLengthMismatchError(
                column_values.len(),
                column_names.len(),
            ));
        }

        // we need to cal self.log() with the correct order of columns, replacing missing columns with an empty string
        let mut new_column_values: Vec<String> = Vec::with_capacity(self.columns.len());

        for column in self.columns.iter() {
            if column_names.contains(column) {
                let i = column_names.iter().position(|x| x == column).unwrap();
                new_column_values.push(column_values[i].clone());
            } else {
                new_column_values.push("".to_string());
            }
        }

        self.log(new_column_values)
    }
}

pub struct BIDSEventLogger {
    logger: CSVEventLogger,
    start_time: std::time::Instant,
}

impl BIDSEventLogger {
    pub fn new<P, I, S>(
        path: P,
        columns: I,
        overwrite: bool,
    ) -> Result<Self, PsychophysicsError>
    where
        P: Into<std::path::PathBuf>,
        I: IntoIterator<Item = S>,
        S: Into<String> + 'static,
    {
        // make sure that the path ends with "events.tsv"
        let path = path.into();
        let path_str = path.to_str().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "path contains invalid characters",
            )
        })?;

        if !path_str.ends_with("events.tsv") {
            return Err(PsychophysicsError::InvalidBIDSPathError(
                path_str.to_string(),
                "path must end with \"*events.tsv\"".to_string(),
            ));
        }

        // add mandatory columns "onset" and "duration"
        let columns: Vec<String> = columns.into_iter().map(Into::into).collect();
        let mandatory_columns = vec!["onset".to_string(), "duration".to_string()];
        let columns: Vec<String> = mandatory_columns.into_iter().chain(columns).collect();

        let logger = CSVEventLogger::new(path, columns, '\t' as u8, overwrite)?;

        Ok(Self {
            logger,
            start_time: std::time::Instant::now(),
        })
    }

    /// Log an event.
    pub fn log<I>(
        &mut self,
        columns_values: I,
        duration: f64,
    ) -> Result<(), PsychophysicsError>
    where
        I: IntoStringVector,
    {
        // convert to vector
        let columns_values: Vec<String> = columns_values.into_string_vec();

        // calculate onset and duration
        let onset = self.start_time.elapsed().as_secs_f64();

        // add onset and duration to event
        let columns_values: Vec<String> = vec![onset.to_string(), duration.to_string()]
            .into_iter()
            .chain(columns_values.into_iter())
            .collect();

        // log event
        self.logger.log(columns_values)
    }

    /// Log an event with the given column names and values.
    pub fn log_cols<I, J>(
        &mut self,
        column_names: I,
        column_values: J,
        duration: f64,
    ) -> Result<(), PsychophysicsError>
    where
        I: IntoStringVector,
        J: IntoStringVector,
    {
        // convert to vector
        let column_names: Vec<String> = column_names.into_string_vec();
        let column_values: Vec<String> = column_values.into_string_vec();

        // calculate onset and duration
        let onset = self.start_time.elapsed().as_secs_f64();

        // add onset and duration to event
        let column_names: Vec<String> = vec!["onset".to_string(), "duration".to_string()]
            .into_iter()
            .chain(column_names.into_iter())
            .collect();

        let column_values: Vec<String> = vec![onset.to_string(), duration.to_string()]
            .into_iter()
            .chain(column_values.into_iter())
            .collect();

        // log event
        self.logger.log_cols(column_names, column_values)
    }
}

pub fn sleep_secs(secs: f64) {
    std::thread::sleep(std::time::Duration::from_secs_f64(secs));
}

pub fn create_random_lowercase_string(len: usize) -> String {
    use rand::Rng;

    let mut rng = rand::thread_rng();

    let random_string: String = (0..len)
        .map(|_| {
            // Generate a random number in the ASCII range of lowercase letters
            let ascii = rng.gen_range(97..=122) as u8;
            // Convert the number to a char
            ascii as char
        })
        .collect();

    random_string
}

/// Converts a value to a byte array that can be used in WebGPU Shading Language (WGSL).
/// For sake of simplicity, t
// pub trait ToWgslBytes<S> {
//     /// Returns the WebGPU Shading Language (WGSL) representation of the object as a byte array.
//     fn to_wgsl_bytes(&self) -> Vec<u8>;
// }

const fn next_power_of_two(mut n: usize) -> usize {
    if n == 0 {
        return 1;
    }
    n -= 1;
    n |= n >> 1;
    n |= n >> 2;
    n |= n >> 4;
    n |= n >> 8;
    n |= n >> 16;
    n += 1;
    n
}

pub trait ToWgslBytes {
    type Bytes;

    fn to_wgsl_bytes(&self) -> Self::Bytes;
}

macro_rules! impl_to_wgsl_bytes {
    ( $( $t:ty ),* ) => {
        $(
            impl ToWgslBytes for $t {
                // use a power of two for the size of the byte array
                type Bytes = [u8; next_power_of_two(std::mem::size_of::<$t>())];

                fn to_wgsl_bytes(&self) -> [u8; next_power_of_two(std::mem::size_of::<$t>())] {
                    let bytes: [u8; std::mem::size_of::<$t>()] = bytemuck::cast(*self);

                     // avoid a copy if the size of the byte array is already a power of two
                    if std::mem::size_of::<$t>() == next_power_of_two(std::mem::size_of::<$t>()) {
                        return bytes;
                    }

                    let mut aligned_bytes = [0; next_power_of_two(std::mem::size_of::<$t>())];

                    aligned_bytes[..bytes.len()].copy_from_slice(&bytes);
                    aligned_bytes
                }
            }
        )*
    };
}

// implement ToWgslBytes for common types
impl_to_wgsl_bytes!(f32, f64, u32, u64, i32, i64, bool);
