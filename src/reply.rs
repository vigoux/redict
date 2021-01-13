use crate::status::Status;
use std::io::BufRead;
use std::str::FromStr;

pub struct Reply {
    pub status: Status,
    pub text: String
}

impl Reply {

    pub fn from_reader<T>(r: &mut T) -> Result<Self, &'static str> where T: BufRead {
        // Assumes that we are actually reading a reply
        let mut iter = r.lines().filter_map(|l| l.ok());

        let firstline = iter.next().ok_or("Nothing to read !")?;
        let (statustxt, text) = firstline.split_at(3);

        let status = Status::from_str(statustxt).map_err(|_| "Could not parse status code")?;

        Ok(Reply {
            status,
            text: String::from(text.trim())
        })
    }
}
