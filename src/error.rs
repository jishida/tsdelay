use ffi::*;
use libc::c_int;
use std::error::Error as StdError;
use std::ffi::CStr;
use std::fmt;
use std::str::from_utf8_unchecked;

#[derive(Debug)]
pub struct AVError {
    errnum: c_int,
}

impl StdError for AVError {}

impl fmt::Display for AVError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = [0i8; AV_ERROR_MAX_STRING_SIZE as usize];
        let ret = unsafe { av_strerror(self.errnum, buf.as_mut_ptr(), AV_ERROR_MAX_STRING_SIZE) };
        let description = if ret == 0 {
            unsafe { from_utf8_unchecked(CStr::from_ptr(buf.as_ptr()).to_bytes()) }
        } else {
            "Unknown AV error"
        };
        write!(f, "{} ({})", description, self.errnum)
    }
}

#[derive(Debug)]
pub struct Error {
    inner: Box<Inner>,
}

#[derive(Debug)]
struct Inner {
    msg: String,
    kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
    AV(AVError),
    Other,
}

impl StdError for Error {
    fn description(&self) -> &str {
        self.inner.msg.as_str()
    }

    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self.inner.kind {
            ErrorKind::AV(ref e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl Error {
    pub fn new<S: Into<String>>(msg: S) -> Error {
        let inner = Inner {
            msg: msg.into(),
            kind: ErrorKind::Other,
        };
        Error {
            inner: Box::new(inner),
        }
    }

    pub fn from_with_errnum<S: Into<String>>(msg: S, errnum: c_int) -> Error {
        let averr = AVError { errnum: errnum };
        let inner = Inner {
            msg: msg.into(),
            kind: ErrorKind::AV(averr),
        };
        Error {
            inner: Box::new(inner),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn empty_result<S: Into<String>>(msg: S, errnum: c_int) -> Result<()> {
    if errnum == 0 {
        Ok(())
    } else {
        Err(Error::from_with_errnum(msg, errnum))
    }
}
