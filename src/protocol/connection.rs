use crate::protocol::reply::{Reply, ParseReplyError};
use crate::protocol::status::{Status, ReplyKind, Category};
use std::io::{Write, BufReader, BufRead, BufWriter};
use super::{Database, Definition, Strategy, Match};
use std::convert::From;
use std::net::TcpStream;

#[derive(Debug)]
pub enum DICTError {
    ReplyError(ParseReplyError),
    SystemError(Reply),
    UnexpectedPacket(DICTPacket),

    // Read / Write things
    NoAnswer,
    ReadWriteError(std::io::Error),
    MalformedAnswer(&'static str),
}

#[derive(Debug)]
pub enum DICTPacketKind {
    // Generic
    ReplyOnly,
    OkReply,

    // TODO: coorectly use the information here
    InitialConnection,

    // DEFINE packets
    DefinitionsFollow,
    Definition(Definition),

    // MATCH packets
    Matches(Vec<Match>),

    // SHOW packets
    Databases(Vec<Database>),
    Strategies(Vec<Strategy>)

    // TODO: There is way more specific packets
}

#[derive(Debug)]
pub struct DICTPacket(pub DICTPacketKind, pub Reply);

pub struct DICTConnection {
    input: BufReader<TcpStream>,
    output: BufWriter<TcpStream>
}

impl DICTConnection {

    pub fn new(inner: TcpStream) -> std::io::Result<Self> {
        let input = BufReader::new(inner.try_clone()?);
        Ok(DICTConnection {
            input,
            output: BufWriter::new(inner)
        })
    }

    fn read_raw_text(&mut self) -> Vec<String> {
        let mut line = String::new();
        let mut text = Vec::with_capacity(10);

        loop {
            if self.input.read_line(&mut line).is_ok() {
                let line_t = line.trim_end_matches("\r\n");
                if line_t.eq(".") {
                    break;
                } else {
                    text.push(line_t.to_owned());
                    line.clear();
                }
            }
        }

        text
    }

    pub fn client(&mut self, client: String) -> Result<Reply, DICTError> {
        writeln!(self.output, "CLIENT \"{}\"", client)?;
        self.output.flush()?;

        match self.next().ok_or(DICTError::NoAnswer)?? {
            DICTPacket(DICTPacketKind::OkReply, r) => Ok(r),
            e => Err(DICTError::UnexpectedPacket(e))
        }
    }

    pub fn define(&mut self, database: Database, word: String)
    -> Result<(Vec<Definition>, Reply), DICTError> {
        writeln!(self.output, "DEFINE \"{}\" \"{}\"", database.name, word)?;
        self.output.flush()?;

        let reply = self.next().ok_or(DICTError::NoAnswer)??;

        // start of answer
        match reply {
            DICTPacket(DICTPacketKind::DefinitionsFollow, _) => {},
            p => {
                return Err(DICTError::UnexpectedPacket(p));
            }
        }

        let mut defs: Vec<Definition> = Vec::new();

        for p in self {
            match p {
                Ok(DICTPacket(DICTPacketKind::Definition(def), _)) => {
                    defs.push(def);
                },
                Ok(DICTPacket(DICTPacketKind::OkReply, _)) => {
                    break;
                }
                Ok(unexp) => {
                    return Err(DICTError::UnexpectedPacket(unexp));
                }
                Err(e) => {
                    return Err(e);
                },
            }
        };

        Ok((defs, reply.1))
    }

    pub fn match_db(&mut self, db: Database, strat: Strategy, word: String)
    -> Result<(Vec<Match>, Reply), DICTError> {
        writeln!(self.output, "MATCH {} {} {}", db.name, strat.name, word)?;
        self.output.flush()?;

        match self.next().ok_or(DICTError::NoAnswer)?? {
            DICTPacket(DICTPacketKind::Matches(matches), r) => {
                let ok = self.next().ok_or(DICTError::NoAnswer)??;

                if let DICTPacket(DICTPacketKind::OkReply, _) = ok {
                    Ok((matches, r))
                } else {
                    Err(DICTError::UnexpectedPacket(ok))
                }
            },
            e => Err(DICTError::UnexpectedPacket(e))
        }
    }

    pub fn show_db(&mut self) -> Result<(Vec<Database>, Reply), DICTError> {
        writeln!(self.output, "SHOW DATABASES")?;
        self.output.flush()?;

        match self.next().ok_or(DICTError::NoAnswer)?? {
            DICTPacket(DICTPacketKind::Databases(dbs), r) => {
                let ok = self.next().ok_or(DICTError::NoAnswer)??;

                if let DICTPacket(DICTPacketKind::OkReply, _) = ok {
                    Ok((dbs, r))
                } else {
                    Err(DICTError::UnexpectedPacket(ok))
                }
            },
            e => Err(DICTError::UnexpectedPacket(e))
        }
    }

    pub fn show_strat(&mut self) -> Result<(Vec<Strategy>, Reply), DICTError> {
        writeln!(self.output, "SHOW STRATEGIES")?;
        self.output.flush()?;

        match self.next().ok_or(DICTError::NoAnswer)?? {
            DICTPacket(DICTPacketKind::Strategies(strats), r) => {
                let ok = self.next().ok_or(DICTError::NoAnswer)??;

                if let DICTPacket(DICTPacketKind::OkReply, _) = ok {
                    Ok((strats, r))
                } else {
                    Err(DICTError::UnexpectedPacket(ok))
                }
            },
            e => Err(DICTError::UnexpectedPacket(e))
        }
    }
}

macro_rules! get_argument {
    ($arguments:ident, $index:literal, $err:expr) => {
        if let Some(arg) = $arguments.get($index) {
            arg
        } else {
            return Some(Err($err));
        }
    };
}

fn parse_cmd_argument(reply_text: &String) -> Vec<String> {
    let mut ret: Vec<String> = Vec::new();
    let mut tmp: String = String::new();

    let mut in_string: bool = false;

    for part in reply_text.split_ascii_whitespace() {
        if !in_string {
            // Starting a string
            if let Some(suffix) = part.strip_prefix('"') {
                if let Some(oneword) = suffix.strip_suffix('"') {
                    // That ends here too
                    ret.push(String::from(oneword));
                } else {
                    in_string = true;

                    tmp.push_str(suffix);
                }
            } else {
                ret.push(String::from(part));
            }
        } else {
            tmp.push_str(" ");
            if let Some(preffix) = part.strip_suffix('"') {
                tmp.push_str(preffix);
                ret.push(tmp);

                in_string = false;
                tmp = String::new();
            } else {
                tmp.push_str(part);
            }
        }
    }

    ret
}

impl Iterator for DICTConnection {
    type Item = Result<DICTPacket, DICTError>;

    fn next(&mut self) -> Option<Self::Item> {
        let reply = match Reply::from_reader(&mut self.input) {
            Ok(rep) => rep,
            Err(e) => { return Some(Err(DICTError::ReplyError(e))); }
        };

        match reply.status {
            // Generic
            Status(
                ReplyKind::PositiveCompletion,
                Category::System,
                0
                ) => {
                Some(Ok(DICTPacket(DICTPacketKind::OkReply, reply)))
            },

            // DEFINE Command
            Status(
                ReplyKind::PositivePreliminary,
                Category::System,
                0
                )=> {
                Some(Ok(DICTPacket(DICTPacketKind::DefinitionsFollow, reply)))
            },
            Status(
                ReplyKind::PositivePreliminary,
                Category::System,
                1
                ) => {
                // Definition

                let arguments = parse_cmd_argument(&reply.text);
                let dbname = get_argument!(arguments, 1, DICTError::MalformedAnswer("Missing database name"));
                let dbdesc = get_argument!(arguments, 2, DICTError::MalformedAnswer("Missing database description"));

                let text = self.read_raw_text();

                let def = Definition {
                    source: Database {
                        name: String::from(dbname),
                        desc: String::from(dbdesc),
                    },
                    text
                };

                Some(Ok(DICTPacket(DICTPacketKind::Definition(def), reply)))
            }

            // MATCH command
            Status(
                ReplyKind::PositivePreliminary,
                Category::System,
                2
                ) => {
                let mut matches: Vec<Match> = Vec::new();
                for match_def in self.read_raw_text() {
                    let arguments = parse_cmd_argument(&match_def);
                    let dbname = get_argument!(arguments, 0, DICTError::MalformedAnswer("Missing database name"));
                    let word = get_argument!(arguments, 1, DICTError::MalformedAnswer("Missing database description"));

                    matches.push(Match {
                        source: Database::from(dbname.to_owned()),
                        word: word.to_owned()
                    });
                }

                Some(Ok(DICTPacket(DICTPacketKind::Matches(matches), reply)))
            },


            // SHOW DB command
            Status(
                ReplyKind::PositivePreliminary,
                Category::Information,
                0
                ) => {
                let mut dbs: Vec<Database> = Vec::new();
                for db_def in self.read_raw_text() {
                    let arguments = parse_cmd_argument(&db_def);
                    let name = get_argument!(arguments, 0, DICTError::MalformedAnswer("Missing database name"));
                    let desc = get_argument!(arguments, 1, DICTError::MalformedAnswer("Missing database description"));

                    dbs.push(Database {
                        name: name.to_owned(),
                        desc: desc.to_owned()
                    });
                }

                Some(Ok(DICTPacket(DICTPacketKind::Databases(dbs), reply)))
            },

            // SHOW STRAT command
            Status(
                ReplyKind::PositivePreliminary,
                Category::Information,
                1
                ) => {
                let mut strats: Vec<Strategy> = Vec::new();
                for strat_def in self.read_raw_text() {
                    let arguments = parse_cmd_argument(&strat_def);
                    let name = get_argument!(arguments, 0, DICTError::MalformedAnswer("Missing database name"));
                    let desc = get_argument!(arguments, 1, DICTError::MalformedAnswer("Missing database description"));

                    strats.push(Strategy {
                        name: name.to_owned(),
                        desc: desc.to_owned()
                    });
                }

                Some(Ok(DICTPacket(DICTPacketKind::Strategies(strats), reply)))
            },
            ref r if r.is_positive() => Some(Ok(DICTPacket(DICTPacketKind::ReplyOnly, reply))),
            _ => Some(Err(DICTError::SystemError(reply)))
        }
    }
}

impl From<ParseReplyError> for DICTError {
    fn from(src: ParseReplyError) -> Self {
        DICTError::ReplyError(src)
    }
}

impl From<std::io::Error> for DICTError {
    fn from(src: std::io::Error) -> Self {
        DICTError::ReadWriteError(src)
    }
}
