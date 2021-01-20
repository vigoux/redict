use dictproto::protocol::{
    Definition,
    Database,
    Strategy,
    Match,
    reply::Reply,
    connection::*
};
use std::convert::Into;

use std::net::TcpStream;

#[derive(Debug, Clone, Copy)]
pub enum AppMode {
    Define,
    Match,
    Strategies,
    Databases
}

impl Into<usize> for AppMode {
    fn into(self) -> usize {
        match self {
            Self::Define => 0,
            Self::Match => 1,
            Self::Strategies => 2,
            Self::Databases => 3
        }
    }
}

impl AppMode {
    pub fn text(&self) -> &'static str {
        match self {
            Self::Define => "Define",
            Self::Match => "Match",
            Self::Strategies => "Strategies",
            Self::Databases => "Databases"
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Self::Define => Self::Match,
            Self::Match => Self::Strategies,
            Self::Strategies => Self::Databases,
            Self::Databases => Self::Define
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            Self::Define => Self::Databases,
            Self::Match => Self::Define,
            Self::Strategies => Self::Match,
            Self::Databases => Self::Strategies
        }
    }

    pub fn values() -> Vec<Self> {
        vec![AppMode::Define, AppMode::Match, AppMode::Strategies, AppMode::Databases]
    }
}

pub struct App {
    pub searched: String,

    // To display things
    pub results: Vec<Definition>,
    pub last_status: Option<Reply>,
    pub databases: Vec<Database>,
    pub stategies: Vec<Strategy>,
    pub matches: Vec<Match>,

    mode: AppMode,
    history: History,
    selected_def: usize,
    scroll_amount: u16,
    conn: DICTConnection
}

const SCROLL_AMOUNT: u16 = 10;

impl App {
    pub fn new(conn: TcpStream) -> Self {
        // First answer
        let mut conn = DICTConnection::new(conn).unwrap();

        // TODO: Maybe things can fail here... Possibly show status on startup ?
        let last_status = if let Some(Ok(rep)) = conn.next() {
            Some(rep.1)
        } else {
            None
        };

        let mut app = App {
            searched: String::new(),
            results: vec![Definition::empty()],
            databases: Vec::new(),
            stategies: Vec::new(),
            matches: Vec::new(),
            last_status,
            history: History::new(),
            conn,
            mode: AppMode::Define,
            selected_def: 0,
            scroll_amount: 0
        };

        app.run_client();
        app.run_show_dbs();
        app.run_show_strats();

        app
    }

    pub fn run_client(&mut self) {
        self.last_status = self.conn.client("redict".to_owned()).ok();
    }

    pub fn run_define(&mut self) {
        self.history.push(self.searched.to_owned());
        self.scroll_amount = 0;
        self.selected_def = 0;

        let answer = self.conn.define(Database::all(), (*self.searched).to_owned());

        match answer {
            Ok((defs, status)) => {
                self.results = defs;
                self.last_status = Some(status);
            },
            Err(DICTError::UnexpectedPacket(DICTPacket(_, r)))
                | Err(DICTError::SystemError(r)) => {
                self.definition_reset();
                self.last_status = Some(r);
            }
            _ => {
                self.definition_reset();
                self.last_status = None;
            }
        }
    }

    pub fn run_match(&mut self) {
        self.history.push(self.searched.to_owned());
        self.scroll_amount = 0;
        self.selected_def = 0;

        let answer = self.conn.match_db(Database::all(), Strategy::default(),(*self.searched).to_owned());

        match answer {
            Ok((matches, status)) => {
                self.matches = matches;
                self.last_status = Some(status);
            },
            Err(DICTError::UnexpectedPacket(DICTPacket(_, r)))
                | Err(DICTError::SystemError(r)) => {
                self.match_reset();
                self.last_status = Some(r);
            }
            _ => {
                self.match_reset();
                self.last_status = None;
            }
        }
    }

    pub fn run_show_dbs(&mut self) {
        self.scroll_amount = 0;
        let answer = self.conn.show_db();

        match answer {
            Ok((dbs, status)) => {
                self.databases = dbs;
                self.last_status = Some(status);
            },
            Err(DICTError::UnexpectedPacket(DICTPacket(_, r)))
                | Err(DICTError::SystemError(r)) => {
                self.last_status = Some(r);
            }
            _ => {
                self.last_status = None;
            }
        }
    }

    pub fn run_show_strats(&mut self) {
        self.scroll_amount = 0;
        let answer = self.conn.show_strat();

        match answer {
            Ok((strats, status)) => {
                self.stategies = strats;
                self.last_status = Some(status);
            },
            Err(DICTError::UnexpectedPacket(DICTPacket(_, r)))
                | Err(DICTError::SystemError(r)) => {
                self.last_status = Some(r);
            }
            _ => {
                self.last_status = None;
            }
        }
    }

    // Definition selection
    pub fn selected_def(&self) -> usize {
        self.selected_def
    }

    pub fn search_reset(&mut self) {
        self.searched.clear();
        self.definition_reset();
    }

    fn definition_reset(&mut self) {
        self.results.truncate(0);
        self.results.push(Definition::empty());
    }

    fn match_reset(&mut self) {
        self.matches.truncate(0);
    }

    pub fn next_definition(&mut self) {
        self.selected_def = (self.selected_def + 1) % self.results.len();
        self.scroll_amount = 0;
    }

    pub fn previous_definition(&mut self) {
        if self.selected_def > 0 {
            self.selected_def = (self.selected_def - 1) % self.results.len();
        } else {
            self.selected_def = self.results.len() - 1;
        }
        self.scroll_amount = 0;
    }

    // Definition scrolling
    pub fn definition_scroll(&self) -> u16 {
        self.scroll_amount
    }

    pub fn scroll_down(&mut self) {
        if (self.scroll_amount as usize)
            < self.results[self.selected_def].text.len() - SCROLL_AMOUNT as usize {
            self.scroll_amount += SCROLL_AMOUNT;
        }
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_amount >= SCROLL_AMOUNT {
            self.scroll_amount -= SCROLL_AMOUNT;
        }
    }

    // Mode
    pub fn mode(&self) -> AppMode {
        self.mode
    }

    pub fn set_mode(&mut self, new: AppMode) {
        self.mode = new;
        self.scroll_amount = 0;
    }

    // History
    pub fn history_goto(&mut self, m: HistoryMovement) {
        self.history.goto(m);
        if let Some(s) = self.history.current() {
            self.searched = s.to_owned();
        } else {
            self.search_reset();
        }
    }
}

struct History {
    current: usize,
    items: Vec<String>
}

pub enum HistoryMovement {
    First,
    Last,
    Previous,
    Next
}

impl History {
    pub fn new() -> Self {
        Self {
            current: 0,
            items: vec![]
        }
    }

    pub fn push(&mut self, item: String) {
        if self.items.len() > 0 {
            self.items.truncate(self.current + 1);
        }

        self.current = self.items.len();

        self.items.push(item);
    }

    pub fn goto(&mut self, m: HistoryMovement) {
        match m {
            HistoryMovement::First => { self.current = 0 },
            HistoryMovement::Last => { self.current = self.items.len() - 1 },
            HistoryMovement::Next => {
                if self.current + 1 < self.items.len() {
                    self.current += 1;
                }
            },
            HistoryMovement::Previous => {
                if self.current > 0 {
                    self.current -= 1;
                }
            }
        }
    }

    pub fn current(&self) -> Option<&String> {
        self.items.get(self.current)
    }
}