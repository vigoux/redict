use std::str::FromStr;
use std::convert::TryFrom;
use std::fmt::Display;

#[derive(Debug, Eq, PartialEq)]
pub enum ParseStatusError {
    InvalidReplyKind,
    InvalidCategory,
    MissingErrNr
}

#[derive(Debug, Eq, PartialEq)]
pub enum ReplyKind {
    PositivePreliminary,
    PositiveCompletion,
    PositiveIntermediate,
    NegativeTransient,
    NegativePermanent
}

impl TryFrom<char> for ReplyKind {
    type Error = ParseStatusError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '1' => Ok(ReplyKind::PositivePreliminary),
            '2' => Ok(ReplyKind::PositiveCompletion),
            '3' => Ok(ReplyKind::PositiveIntermediate),
            '4' => Ok(ReplyKind::NegativeTransient),
            '5' => Ok(ReplyKind::NegativePermanent),
            _ => Err(ParseStatusError::InvalidReplyKind)
        }
    }
}

impl Display for ReplyKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            ReplyKind::PositivePreliminary => '1',
            ReplyKind::PositiveCompletion => '2',
            ReplyKind::PositiveIntermediate => '3',
            ReplyKind::NegativeTransient => '4',
            ReplyKind::NegativePermanent => '5'
        })
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Category {
    Syntax,
    Information,
    Connection,
    Authentication,
    Unspecified,
    System,
    Nonstandard
}

impl TryFrom<char> for Category {
    type Error = ParseStatusError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '0' => Ok(Category::Syntax),
            '1' => Ok(Category::Information),
            '2' => Ok(Category::Connection),
            '3' => Ok(Category::Authentication),
            '4' => Ok(Category::Unspecified),
            '5' => Ok(Category::System),
            '8' => Ok(Category::Nonstandard),
            _ => Err(ParseStatusError::InvalidCategory)
        }
    }
}

impl Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Category::Syntax => '0',
            Category::Information => '1',
            Category::Connection => '2',
            Category::Authentication => '3',
            Category::Unspecified => '4',
            Category::System => '5',
            Category::Nonstandard => '8'
        })
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Status(pub ReplyKind, pub Category, pub u8);

impl FromStr for Status {
    type Err = ParseStatusError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let chars: Vec<char> = s.chars().collect();

        let reply = chars.get(0).ok_or(ParseStatusError::InvalidReplyKind)?;
        let category = chars.get(1).ok_or(ParseStatusError::InvalidCategory)?;

        let errnr: u8 = if let Some(ref c) = chars.get(2) {
            let tmp = c.to_digit(10).ok_or(ParseStatusError::MissingErrNr)?;
            u8::try_from(tmp).map_err(|_| ParseStatusError::MissingErrNr)?
        } else {
            return Err(ParseStatusError::MissingErrNr);
        };

        Ok(Status(ReplyKind::try_from(*reply)?, Category::try_from(*category)?, errnr))
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}", self.0, self.1, self.2)
    }
}

impl Status {
    pub fn is_positive(&self) -> bool {
        match self.0 {
            ReplyKind::PositiveCompletion
                | ReplyKind::PositiveIntermediate
                | ReplyKind::PositivePreliminary => true,
            _ => false
        }
    }

    pub fn is_start(&self) -> bool {
        *self == Status(ReplyKind::PositiveCompletion, Category::Connection, 0)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic_parsing() {
        let ok = Status::from_str("250").unwrap();

        assert_eq!(ok, Status(ReplyKind::PositiveCompletion, Category::System, 0));
    }

    #[test]
    fn invalid_reply() {
        if let Err(ParseStatusError::InvalidReplyKind) = Status::from_str("700") {
            // Do nothing
        } else {
            panic!();
        }
    }

    #[test]
    fn invalid_category() {
        if let Err(ParseStatusError::InvalidCategory) = Status::from_str("170") {
            // Do nothing
        } else {
            panic!();
        }
    }

    #[test]
    fn missing_errno() {
        if let Err(ParseStatusError::MissingErrNr) = Status::from_str("17") {
            // Do nothing
        } else {
            panic!();
        }
    }
}
