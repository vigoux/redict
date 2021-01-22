use std::default::Default;

use tui::{Frame, backend::Backend, style::Style, widgets::Borders};
use tui::style::Color;
use tui::widgets::{Paragraph, Block};
use tui::text::{Span, Spans};
use tui::layout::Rect;
use unicode_segmentation::UnicodeSegmentation;

pub enum CursorDirection {
    Left,
    Right
}

pub struct SearchBar {
    text: String,
    cursor: usize
}

impl Default for SearchBar {
    fn default() -> Self {
        Self {
            text: String::new(),
            cursor: 0
        }
    }
}

impl SearchBar {
    pub fn draw<B: Backend>(&self, f: &mut Frame<B>, rect: Rect) {

        let (before_cursor, at_cursor, after_cursor) : (&str, &str, &str) = match self.cursor_indices() {
            (Some(cursor_index), Some(after_cursor_index)) => {
                (self.text.get(..cursor_index).unwrap(),
                self.text.get(cursor_index..after_cursor_index).unwrap(),
                self.text.get(after_cursor_index..).unwrap())
            },
            (Some(cursor_index), None) => (self.text.get(..cursor_index).unwrap(), self.text.get(cursor_index..).unwrap(), ""),
            (None, None) => (self.text.as_ref(), " ", ""),
            _ => unreachable!()
        };

        let spans = Spans::from(vec![
            Span::raw(before_cursor),
            Span::styled(at_cursor, Style::default().fg(Color::Black).bg(Color::White)),
            Span::raw(after_cursor)
        ]);

        let par = Paragraph::new(spans).block(Block::default().borders(Borders::ALL));
        f.render_widget(par, rect);
    }

    fn cursor_indices(&self) -> (Option<usize>, Option<usize>) {
        let mut graphemes = self.text.grapheme_indices(true);
        if let Some((cursor_index, _)) = graphemes.nth(self.cursor) {
            if let Some((after_cursor_index, _)) = graphemes.next() {
                (Some(cursor_index), Some(after_cursor_index))
            } else {
                (Some(cursor_index), None)
            }
        } else {
            (None, None)
        }
    }

    fn graphemes_nr(&self) -> usize {
        self.text.graphemes(true).count()
    }

    pub fn move_cursor(&mut self, dir: CursorDirection) {
        match dir {
            CursorDirection::Left => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
            },
            CursorDirection::Right => {
                if self.cursor < self.graphemes_nr() {
                    self.cursor += 1;
                }
            }
        }
    }

    pub fn edit(&mut self, c: char) {
        if let (Some(cursor_start), _) = self.cursor_indices() {
            self.text.insert(cursor_start, c);
        } else {
            self.text.push(c);
        }
        self.move_cursor(CursorDirection::Right);
    }

    pub fn delete(&mut self) {
        if self.cursor == 0 { return; }

        self.move_cursor(CursorDirection::Left);
        match self.cursor_indices() {
            (Some(cursor_start), Some(after_cursor_start)) => {
                self.text.replace_range(cursor_start..after_cursor_start, "");
            },
            (Some(cursor_start), None) => {
                self.text.replace_range(cursor_start.., "");
            },
            (None, None) => { self.text.pop(); }
            _ => { return; }
        }
    }

    pub fn text(&self) -> &String {
        &self.text
    }

    pub fn set_text<T>(&mut self, text: &T)
    where T: ToString {
        self.text = text.to_string();
        self.cursor = 0;
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic_insert() {
        let mut search = SearchBar::default();
        search.edit('f'); search.edit('g'); search.edit('h');

        assert_eq!(search.text(), "fgh");
    }

    #[test]
    fn middle_insert() {
        let mut search = SearchBar::default();
        search.edit('f'); search.edit('h');

        search.move_cursor(CursorDirection::Left);
        search.edit('g');

        assert_eq!(search.text(), "fgh");
    }

    #[test]
    fn start_insert() {
        let mut search = SearchBar::default();
        search.edit('g');
        search.move_cursor(CursorDirection::Left);
        search.edit('f');

        assert_eq!(search.text(), "fg");
    }

    #[test]
    fn basic_delete() {
        let mut search = SearchBar::default();
        search.set_text(&"fg");
        search.move_cursor(CursorDirection::Right);
        search.move_cursor(CursorDirection::Right);
        search.delete();

        assert_eq!(search.text(), "f");
    }

    #[test]
    fn middle_delete() {
        let mut search = SearchBar::default();
        search.set_text(&"fg");
        search.move_cursor(CursorDirection::Right);
        search.delete();

        assert_eq!(search.text(), "g");
    }

    #[test]
    fn start_delete() {
        let mut search = SearchBar::default();
        search.set_text(&"fg");
        search.delete();

        assert_eq!(search.text(), "fg");
    }

    #[test]
    fn delete_empty() {
        let mut search = SearchBar::default();
        search.delete();
        // Only assert aliveness
    }

    #[test]
    fn complex_grapheme_insert() {
        let mut search = SearchBar::default();
        search.edit('ĝ');
        search.edit('h');

        assert_eq!(search.text(), "ĝh");
    }

    #[test]
    fn complex_grapheme_delete() {
        let mut search = SearchBar::default();
        search.edit('ĝ');
        search.delete();

        assert!(search.text().is_empty());
    }

    #[test]
    fn clear() {
        let mut search = SearchBar::default();
        search.set_text(&"foo");
        assert_eq!(search.text(), "foo");
        search.clear();
        assert_eq!(search.text(), "");
    }
}
