use crate::searchbar::SearchBar;

use dictproto::{
    Definition,
    Database,
    Strategy,
    Match,
    url::{DICTUrl, DICTUrlAccess},
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
    pub searched: SearchBar,

    // To display things
    pub results: Vec<Definition>,
    pub last_status: Option<Reply>,
    pub databases: Vec<Database>,
    pub stategies: Vec<Strategy>,
    pub matches: Vec<Match>,

    msg_id: String,
    mode: AppMode,
    history: History,
    selected_def: usize,
    scroll_amount: u16,
    conn: DICTConnection
}

fn parse_search_bar(src: &String) -> (String, Database, Strategy) {
    let mut word: String = String::with_capacity(src.len());
    let mut db: Option<Database> = None;
    let mut strat: Option<Strategy> = None;

    for part in src.split_whitespace() {
        match part.get(0..1) {
            Some("@") if db.is_none() => {
                db = Some(Database::from(part.get(1..).unwrap().to_string()));
            },
            Some(":") if strat.is_none() => {
                strat = Some(Strategy::from(part.get(1..).unwrap().to_string()));
            }
            Some(_) => { 
                if !word.is_empty() {
                    word.push_str(" ");
                }
                word.push_str(part); },
            None => {}
        }
    }

    (word,
     db.or(Some(Database::all())).unwrap(),
     strat.or(Some(Strategy::default())).unwrap())
}

const SCROLL_AMOUNT: u16 = 10;

impl App {
    pub fn new(addr: &str) -> Self {
        // Should have been checked in main
        let url = DICTUrl::new(addr).unwrap();
        let stream = TcpStream::connect((url.host, url.port)).expect("Invalid socket address");
        let mut conn = DICTConnection::new(stream).unwrap();

        // TODO: Maybe things can fail here... Possibly show status on startup ?
        let (msg_id, last_status) = conn.start().unwrap();

        let mut app = App {
            searched: SearchBar::default(),
            results: vec![Definition::empty()],
            databases: Vec::new(),
            stategies: Vec::new(),
            matches: Vec::new(),
            msg_id,
            last_status: Some(last_status),
            history: History::new(),
            conn,
            mode: AppMode::Define,
            selected_def: 0,
            scroll_amount: 0
        };

        app.run_client();
        app.run_show_dbs();
        app.run_show_strats();

        // Now use the url
        match url.access_method {
            DICTUrlAccess::Define(word, db, _) => {
                app.searched.set_text(&word);
                app.define_internal(word, db);
            }
            DICTUrlAccess::Match(word, db, strat, _) => {
                app.searched.set_text(&word);
                app.match_internal(word, db, strat);
            },
            _ => {}
        }

        app
    }

    pub fn run_client(&mut self) {
        self.last_status = self.conn.client("redict".to_owned()).ok();
    }

    fn define_internal(&mut self, word: String, db: Database) {
        self.history.push(self.searched.text().to_owned());
        self.scroll_amount = 0;
        self.selected_def = 0;

        let answer = self.conn.define(db, word);

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

    pub fn run_define(&mut self) {
        let (word, db, _) = parse_search_bar(self.searched.text());
        self.define_internal(word, db);
    }

    fn match_internal(&mut self, word: String, db: Database, strat: Strategy) {
        self.history.push(self.searched.text().to_owned());
        self.scroll_amount = 0;
        self.selected_def = 0;

        let answer = self.conn.match_db(db, strat, word);

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

    pub fn run_match(&mut self) {
        let (word, db, strat) = parse_search_bar(self.searched.text());
        self.match_internal(word, db, strat);
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
        let max_scroll = match self.mode {
            AppMode::Define => self.results[self.selected_def].text.len(),
            AppMode::Match => self.matches.len(),
            _ => 0
        };

        if ((self.scroll_amount  + SCROLL_AMOUNT) as usize) < max_scroll {
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
            self.searched.set_text(s);
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
