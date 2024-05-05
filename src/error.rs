use std::{fmt, io};

pub struct Error {
	message: String,
}

impl fmt::Display for Error {
	fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(fmt, "{}", self.message)
	}
}

impl fmt::Debug for Error {
	fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(fmt, "{}", self.message)
	}
}

impl From<io::Error> for Error {
	fn from(value: io::Error) -> Self {
		Error { message: value.to_string() }
	}
}

impl From<&str> for Error {
	fn from(value: &str) -> Self {
		Error { message: value.into() }
	}
}

impl From<String> for Error {
	fn from(value: String) -> Self {
		Error { message: value }
	}
}
