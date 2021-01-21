use std::error::Error;
use std::fmt::{Formatter, Display};
use std::convert::From;
use std::str::FromStr;
use crate::{Database, Strategy};
use url::{Url, ParseError};

#[derive(Debug)]
pub enum DICTUrlError {
    ParseError(ParseError),
    UnknownAccess(String),
    MissingParameters,
    MissingHost,
    Unsupported(&'static str)
}

impl Display for DICTUrlError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::ParseError(_) => "Parse error",
            Self::Unsupported(_) => "Unsuported",
            Self::UnknownAccess(_) => "Unknown access method",
            Self::MissingParameters => "Missing parameters",
            Self::MissingHost => "Missing host"
        })
    }
}

impl From<ParseError> for DICTUrlError {
    fn from(inner: ParseError) -> Self {
        DICTUrlError::ParseError(inner)
    }
}

impl Error for DICTUrlError {
}

pub enum DICTUrlAccess {
    AccessOnly,
    Define(String, Database, Option<usize>),
    Match(String, Database, Strategy, Option<usize>)
}

impl FromStr for DICTUrlAccess {
    type Err = DICTUrlError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let path = if let Some(p) = s.strip_prefix("/") {
            p
        } else {
            return Ok(DICTUrlAccess::AccessOnly);
        };

        let mut parts = path.split(':');

        match parts.next() {
            None | Some("") => Ok(DICTUrlAccess::AccessOnly),
            Some("d") => {
                let word = match parts.next() {
                    Some(w) if !w.is_empty() => w.to_string(),
                    _ => { return Err(DICTUrlError::MissingParameters); }
                };

                let db = match parts.next() {
                    Some(d) if !d.is_empty() => Database::from(d.to_string()),
                    _ => Database::default()
                };

                let nr = if let Some(maybe_n) = parts.next() {
                    if let Ok(n) = maybe_n.parse::<usize>() {
                        Some(n)
                    } else {
                        Some(0)
                    }
                } else {
                    None
                };

                Ok(DICTUrlAccess::Define(word, db, nr))
            },
            Some("m") => {
                let word = match parts.next() {
                    Some(w) if !w.is_empty() => w.to_string(),
                    _ => { return Err(DICTUrlError::MissingParameters); }
                };

                let db = match parts.next() {
                    Some(d) if !d.is_empty() => Database::from(d.to_string()),
                    _ => Database::default()
                };

                let strat = match parts.next() {
                    Some(s) if !s.is_empty() => Strategy::from(s.to_string()),
                    _ => Strategy::default()
                };

                let nr = if let Some(maybe_n) = parts.next() {
                    if let Ok(n) = maybe_n.parse::<usize>() {
                        Some(n)
                    } else {
                        Some(0)
                    }
                } else {
                    None
                };

                Ok(DICTUrlAccess::Match(word, db, strat, nr))
            }
            Some(s) => Err(DICTUrlError::UnknownAccess(s.to_string()))
        }
    }
}

pub struct DICTUrl {
    pub host: String,
    pub port: u16,
    pub access_method: DICTUrlAccess,
}

impl DICTUrl {
    pub fn new(src: &str) -> Result<Self, DICTUrlError> {
        let raw_url = Url::parse(src)?;

        if !raw_url.username().is_empty() {
            return Err(DICTUrlError::Unsupported("Auth part is not supported"));
        }

        let host: String = raw_url.host_str().ok_or(DICTUrlError::MissingHost)?.to_string();
        let port: u16 = raw_url.port().or(Some(2628)).unwrap();
        let access_method = DICTUrlAccess::from_str(raw_url.path())?;

        Ok(DICTUrl { host, port, access_method })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic_parsing() {
        let url = DICTUrl::new("dict://dict.org/d:shortcake:").unwrap();

        assert_eq!(url.host, "dict.org");
        assert_eq!(url.port, 2628);

        if let DICTUrlAccess::Define(word, _, _) = url.access_method {
            assert_eq!(word, String::from("shortcake"));
        } else {
            panic!("Did not return correct access method");
        }
    }
}
