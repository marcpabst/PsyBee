// Copyright (c) 2024 Marc Pabst
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum psydkError {
    // Pyo3 errors
    #[error("{0}")]
    Pyo3Error(#[from] pyo3::PyErr),

    // a brush creation error
    #[error("{0}")]
    BrushError(String),

    // file errors
    #[error("{0}")]
    IOError(#[from] std::io::Error),
    #[error("File already exists and is not empty: {0}")]
    FileExistsAndNotEmptyError(String),

    // event logging errors
    #[error("The column names provided need to be unique")]
    ColumnNamesNotUniqueError,
    #[error("The lenght of the data ({0}) does not match the length of the column names ({1}).")]
    DataLengthMismatchError(usize, usize),
    #[error("The column name {0} does not exist.")]
    ColumnNameDoesNotExistError(String),
    // #[error("{0}")]
    // CSVError(#[from] csv::Error),

    // BIDS errors
    #[error("The provided file name or path is not allowed under the BIDS specification: {0}. Reason: {1}")]
    InvalidBIDSPathError(String, String),

    // custom errors
    #[error("{0}")]
    CustomError(String),

    // image errors
    #[error("{0}")]
    ImageError(#[from] image::ImageError),

    // wrong dimensions
    #[error("The dimensions of the image are not correct. Expected: ({0}, {1}), got: ({2}, {3})")]
    WrongDimensionsError(u32, u32, u32, u32),

    // dimensionsa are not identical
    #[error("The dimensions of the images are not identical. The first image has dimensions: ({0}, {1}). All other images need to have the same dimensions.")]
    NonIdenticalDimensionsError(u32, u32),

    // empty vector provided
    #[error("The provided vector is empty.")]
    EmptyVectorError,

    // // index out of bounds
    // #[error("The index {0} is out of bounds for an array or vector of length {1}.")]
    // IndexOutOfBoundsError(usize, usize),

    // single image error
    #[error("Only one image was provided. This is currently not supported.")]
    SingleImageError,
}

// macro that error with the given message
#[macro_export]
macro_rules! error {
    ($msg:expr) => {
        return Err(psydkError::CustomError($msg.to_string()));
    };
}

// allow psydkError to be converted to a PyErr
impl From<psydkError> for pyo3::PyErr {
    fn from(err: psydkError) -> pyo3::PyErr {
        pyo3::exceptions::PyException::new_err(err.to_string())
    }
}
