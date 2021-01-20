use crate::protocol::status::{Status, ParseStatusError};
use std::io::BufRead;
use std::str::FromStr;
use std::string::ToString;

use std::fmt::Display;
use std::error::Error;

#[derive(Debug)]
pub enum ParseReplyError {
    Status(ParseStatusError),
    FailedToRead
}

impl Display for ParseReplyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ParseReplyError {
    fn description(&self) -> &str {
        match self {
            Self::Status(_) => "An error when parsing status",
            Self::FailedToRead => "Impossible to read"
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        if let Self::Status(inner) = self {
            Some(inner)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub struct Reply {
    pub status: Status,
    pub text: String
}

impl Reply {
    pub fn from_reader<T>(r: &mut T) -> Result<Self, ParseReplyError> where T: BufRead {
        // Assumes that we are actually reading a reply
        let mut iter = r.lines().filter_map(|l| l.ok());

        let firstline = iter.next().ok_or(ParseReplyError::FailedToRead)?;

        Self::from_line(firstline)
    }

    pub fn from_line(line: String) -> Result<Self, ParseReplyError> {
        if line.len() < 3 {
            return Err(ParseReplyError::FailedToRead);
        }
        let (statustxt, text) = line.split_at(3);

        let status = Status::from_str(statustxt).map_err(|e| ParseReplyError::Status(e))?;

        Ok(Reply {
            status,
            text: String::from(text.trim())
        })
    }
}

impl ToString for Reply {
    fn to_string(&self) -> String {
        String::from(
            format!("{} {}", self.status, self.text)
            )
    }
}
