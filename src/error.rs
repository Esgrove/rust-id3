use crate::stream::encoding::Encoding;
use crate::tag::Tag;
use std::error;
use std::fmt;
use std::io;
use std::string;

/// Type alias for the result of tag operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Takes a tag result and maps any partial tag to Ok. An ok result is left untouched. An Err
/// without partial tag is returned as the initial error.
///
/// # Example
/// ```
/// use id3::{Tag, Error, ErrorKind, partial_tag_ok};
///
/// let rs = Err(Error{
///     kind: ErrorKind::Parsing,
///     description: "frame 12 could not be decoded".to_string(),
///     partial_tag: Some(Tag::new()),
/// });
/// assert!(partial_tag_ok(rs).is_ok());
/// ```
pub fn partial_tag_ok(rs: Result<Tag>) -> Result<Tag> {
    match rs {
        Ok(tag) => Ok(tag),
        Err(Error {
            partial_tag: Some(tag),
            ..
        }) => Ok(tag),
        Err(err) => Err(err),
    }
}

/// Takes a tag result and maps the NoTag kind to None. Any other error is returned as Err.
///
/// # Example
/// ```
/// use id3::{Tag, Error, ErrorKind, no_tag_ok};
///
/// let rs = Err(Error{
///     kind: ErrorKind::NoTag,
///     description: "the file contains no ID3 tag".to_string(),
///     partial_tag: None,
/// });
/// assert!(matches!(no_tag_ok(rs), Ok(None)));
///
/// let rs = Err(Error{
///     kind: ErrorKind::Parsing,
///     description: "frame 12 could not be decoded".to_string(),
///     partial_tag: None,
/// });
/// assert!(no_tag_ok(rs).is_err());
/// ```
pub fn no_tag_ok(rs: Result<Tag>) -> Result<Option<Tag>> {
    match rs {
        Ok(tag) => Ok(Some(tag)),
        Err(Error {
            kind: ErrorKind::NoTag,
            ..
        }) => Ok(None),
        Err(err) => Err(err),
    }
}

/// Kinds of errors that may occur while performing metadata operations.
#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    /// An error kind indicating that an IO error has occurred. Contains the original io::Error.
    #[error("IO: {0}")]
    Io(io::Error),
    /// An error kind indicating that a string decoding error has occurred. Contains the invalid
    /// bytes.
    #[error("StringDecoding")]
    StringDecoding(Vec<u8>),
    /// An error kind indicating that the reader does not contain an ID3 tag.
    #[error("NoTag")]
    NoTag,
    /// An error kind indicating that parsing of some binary data has failed.
    #[error("Parsing")]
    Parsing,
    /// An error kind indicating that a specific frame failed to parse.
    #[error("{0}")]
    FrameParsing(FrameError),
    /// An error kind indicating that some input to a function was invalid.
    #[error("InvalidInput")]
    InvalidInput,
    /// An error kind indicating that a feature is not supported.
    #[error("UnsupportedFeature")]
    UnsupportedFeature,
}

/// Structured error for frame-level parsing failures.
#[derive(Debug, Clone, thiserror::Error)]
#[error("frame '{frame_id}' field '{field}': {kind}")]
pub struct FrameError {
    /// The frame ID (e.g. "UFID", "GEOB", "APIC").
    pub frame_id: String,
    /// The field within the frame that failed to parse (e.g. "owner_identifier", "mime_type").
    pub field: String,
    /// The specific kind of frame parsing failure.
    pub kind: FrameErrorKind,
}

/// Specific kinds of frame-level parsing failures.
#[derive(Debug, Clone, thiserror::Error)]
pub enum FrameErrorKind {
    /// A null-terminated string field had no null delimiter.
    #[error(
        "delimiter not found (encoding={encoding:?}, remaining_bytes={remaining_bytes}, \
         hex=[{hex_preview}], ascii=[{ascii_preview}])"
    )]
    DelimiterNotFound {
        /// The text encoding that was being used.
        encoding: Encoding,
        /// Number of remaining bytes in the frame data.
        remaining_bytes: usize,
        /// Hex preview of the remaining data (up to 64 bytes).
        hex_preview: String,
        /// ASCII preview of the remaining data (up to 64 bytes).
        ascii_preview: String,
    },
    /// Other frame parsing failure.
    #[error("{0}")]
    Other(String),
}

/// A structure able to represent any error that may occur while performing metadata operations.
pub struct Error {
    /// The kind of error.
    pub kind: ErrorKind,
    /// A human readable string describing the error.
    pub description: String,
    /// If any, the part of the tag that was able to be decoded before the error occurred.
    pub partial_tag: Option<Tag>,
}

impl Error {
    /// Creates a new `Error` using the error kind and description.
    pub fn new(kind: ErrorKind, description: impl Into<String>) -> Error {
        Error {
            kind,
            description: description.into(),
            partial_tag: None,
        }
    }

    /// Creates a new `Error` from a [`FrameError`].
    ///
    /// The description is automatically derived from the structured error.
    pub fn frame_parsing(error: FrameError) -> Error {
        let description = error.to_string();
        Error {
            kind: ErrorKind::FrameParsing(error),
            description,
            partial_tag: None,
        }
    }

    /// Creates a new `Error` using the error kind and description.
    pub(crate) fn with_tag(self, tag: Tag) -> Error {
        Error {
            partial_tag: Some(tag),
            ..self
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self.kind {
            ErrorKind::Io(ref err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error {
            kind: ErrorKind::Io(err),
            description: "".to_string(),
            partial_tag: None,
        }
    }
}

impl From<string::FromUtf8Error> for Error {
    fn from(err: string::FromUtf8Error) -> Error {
        Error {
            kind: ErrorKind::StringDecoding(err.into_bytes()),
            description: "data is not valid utf-8".to_string(),
            partial_tag: None,
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.description.is_empty() {
            write!(f, "{:?}", self.kind)
        } else {
            write!(f, "{:?}: {}", self.kind, self.description)
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.description.is_empty() {
            write!(f, "{}", self.kind)
        } else {
            write!(f, "{}: {}", self.kind, self.description)
        }
    }
}
