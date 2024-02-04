// Copyright (c) 2024 Marc Pabst
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::loop_frames;
use crate::prelude::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PsychophysicsError {
    // file errors
    #[error("{0}")]
    IOError(#[from] std::io::Error),
    #[error("File already exists and is not empty: {0}")]
    FileExistsAndNotEmptyError(String),

    // event logging errors
    #[error("The column names provided need to be unique")]
    ColumnNamesNotUniqueError,
    #[error(
        "The lenght of the data ({0}) does not match the length of the column names ({1})."
    )]
    DataLengthMismatchError(usize, usize),
    #[error("The column name {0} does not exist.")]
    ColumnNameDoesNotExistError(String),
    #[error("{0}")]
    CSVError(#[from] csv::Error),

    // BIDS errors
    #[error("The provided file name or path is not allowed under the BIDS specification: {0}. Reason: {1}")]
    InvalidBIDSPathError(String, String),

    // custom errors
    #[error("{0}")]
    CustomError(String),

    // serial port errors
    #[cfg(feature = "serial")]
    #[error("{0}")]
    SerialPortError(#[from] serialport::Error),
}

// macro that error with the given message
#[macro_export]
macro_rules! error {
    ($msg:expr) => {
        return Err(PsychophysicsError::CustomError($msg.to_string()));
    };
}
