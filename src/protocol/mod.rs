pub mod connection;
pub mod reply;
pub mod status;
pub mod url;

use std::convert::From;
use std::default::Default;

#[derive(Debug)]
pub struct Database {
    pub name: String,
    pub desc: String
}

impl From<String> for Database {
    fn from(src: String) -> Self {
        Self {
            name: src,
            desc: String::new()
        }
    }
}

impl Database {
    pub fn all() -> Self {
        Database {
            name: String::from("*"),
            desc: String::from("All databases")
        }
    }

    pub fn first() -> Self {
        Database {
            name: String::from("!"),
            desc: String::from("All databases (first match)")
        }
    }
}

impl Default for Database {
    fn default() -> Self {
        Self::first()
    }
}

#[derive(Debug)]
pub struct Definition {
    pub source: Database,
    pub text: Vec<String>
}

impl Definition {
    pub fn empty() -> Self {
        Definition {
            source: Database::all(),
            text: vec![String::from("No definition")]
        }
    }
}

#[derive(Debug)]
pub struct Strategy {
    pub name: String,
    pub desc: String
}

impl From<String> for Strategy {
    fn from(src: String) -> Self {
        Strategy {
            name: src,
            desc: String::new()
        }
    }
}

impl Strategy {
    pub fn exact() -> Self {
        Self::from(String::from("exact"))
    }

    pub fn prefix() -> Self {
        Self::from(String::from("prefix"))
    }
}

impl Default for Strategy {
    fn default() -> Self {
        Self {
            name: String::from("."),
            desc: String::from("Server default")
        }
    }
}

#[derive(Debug)]
pub struct Match {
    pub source: Database,
    pub word: String
}
