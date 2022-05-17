use std::io;

use cafebabe::ParseError;
use thiserror::Error;
use zip::result::ZipError;

use crate::descriptor::DescriptorError;

pub type Result<A, E = Error> = std::result::Result<A, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    ClassError(ParseError),
    #[error("archive error: {0}")]
    ArchiveError(#[from] ZipError),
    #[error("method descriptor error: {0}")]
    DescriptorError(#[from] DescriptorError),
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),
    #[error("too many matches for pattern {0}")]
    TooManyMatches(usize),
    #[error("pattern {0} not found")]
    PatternNotFound(usize),
}
