extern crate pancurses;
extern crate textwrap;

use pancurses::{initscr, endwin, Input, Attribute};
use pancurses::{COLOR_PAIR, COLOR_WHITE, COLOR_BLUE};
use std::borrow::Cow;
use std::cmp::{max, min};
use std::ops::Deref;
use std::fmt::Display;
use textwrap::Wrapper;

pub type Message = String;
const CHROME_HEIGHT: usize = 3; // the number of lines we need for the ui (input area and topic)


pub struct Buffer {
    backlog: Vec<String>,
    input: Vec<char>,
    position: usize,
    prompt: Cow<'static, str>,
    topic: Cow<'static, str>,
}

impl Buffer {
    fn new() -> Buffer {
        Buffer {
            backlog: Vec::new(),
            input: Vec::new(),
            position: 0,
            prompt: "".into(),
            topic: "".into(),
        }
    }
}

pub struct Window {
    win: pancurses::Window,
    buffers: Vec<Buffer>,
    cur_buf: usize,
    max_y: i32,
    max_x: i32,
    /// catch the next key and add a debug representation to input
    catch_key: bool,
    /// initiate a buffer switch
    /// Some(None) means next key is the number
    /// Some(Some(Vec<_>)) means we wait for two keys
    navigate_buffer: Option<Option<String>>,
}

impl Window {
    pub fn initscr() -> Window {
        let win = initscr();
        win.timeout(100);
        win.keypad(true);

        pancurses::start_color();
        pancurses::use_default_colors();

        pancurses::init_pair(1, COLOR_WHITE, COLOR_BLUE);

        pancurses::noecho();

        let (max_y, max_x) = win.get_max_yx();

        Window {
            win,
            buffers: vec![Buffer::new(), Buffer::new()],
            cur_buf: 0,
            max_y,
            max_x,
            catch_key: false,
            navigate_buffer: None,
        }
    }

    pub fn writeln<D: Display>(&mut self, txt: D) {
        self.cur_buf_mut().backlog.push(txt.to_string());
        self.redraw();
    }

    pub fn redraw(&self) {
        self.win.erase();
        self.draw_topic();

        let max_lines = self.max_y as usize - CHROME_HEIGHT;
        let mut backlog = Vec::new();

        // leave some space for edgecases
        let wrapper = Wrapper::new((self.max_x - 3) as usize)
                        .subsequent_indent("| ");

        // fill the screen bottom to top
        for line in self.cur_buf().backlog.iter().rev() {
            // we have to insert the last line first since we are writing bottom to top
            for l in wrapper.wrap(line).iter().rev() {
                backlog.push(format!("{}\n", l));
            }

            // if the screen is full, break early
            if backlog.len() >= max_lines {
                break;
            }
        }

        // remove excess lines before printing the screen
        while backlog.len() > max_lines {
            backlog.pop();
        }

        // process the backlog in reverse order to write from top to bottom
        for line in backlog.iter().rev() {
            self.win.addstr(&line);
        }

        self.draw_input();
        self.win.refresh();
    }

    pub fn draw_topic(&self) {
        // this is needed to handle some utf8 edgecases 🤷
        let topic = self.cur_buf().topic.chars().take(self.max_x as usize).collect::<String>();
        let topic_len = self.cur_buf().topic.chars().count() as i32;

        self.win.attrset(COLOR_PAIR(1));
        self.win.addstr(topic);
        self.win.hline(' ', self.max_x - topic_len);
        self.win.attrset(Attribute::Normal);
        self.win.mv(1, 0);
    }

    pub fn draw_input(&self) {
        self.win.mv(self.max_y -2, 0);

        self.win.attrset(COLOR_PAIR(1));
        self.win.hline(' ', self.max_x);
        self.win.attrset(Attribute::Normal);

        self.win.mv(self.max_y -1, 0);
        self.win.addstr(&self.cur_buf().prompt);
        for x in self.input() {
            self.win.addch(*x);
        }
        self.win.mv(self.max_y -1, self.cursor_pos());
    }

    pub fn resize(&mut self) {
        let (max_y, max_x) = self.win.get_max_yx();
        self.max_y = max_y;
        self.max_x = max_x;
        self.redraw();
    }

    fn cursor_pos(&self) -> i32 {
        (self.cur_buf().position + self.cur_buf().prompt.len()) as i32
    }

    pub fn cur_buf(&self) -> &Buffer {
        self.buffers.get(self.cur_buf).unwrap()
    }

    pub fn cur_buf_mut(&mut self) -> &mut Buffer {
        self.buffers.get_mut(self.cur_buf).unwrap()
    }

    pub fn input(&self) -> &Vec<char> {
        &self.cur_buf().input
    }

    pub fn input_mut(&mut self) -> &mut Vec<char> {
        &mut self.cur_buf_mut().input
    }

    pub fn try_navigate(&mut self, window: String) {
        if let Ok(mut idx) = window.parse() {
            if idx == 0 {
                idx = 10
            }
            self.navigate(idx - 1);
        }
    }

    pub fn navigate(&mut self, window: usize) {
        if self.buffers.get(window).is_some() {
            self.cur_buf = window;
            self.redraw();
        }
    }

    pub fn get(&mut self) -> Option<String> {
        match self.win.getch() {
            Some(c) if self.catch_key => {
                let x = format!("{:?}", c);
                self.cur_buf_mut().input.extend(x.chars());
                self.cur_buf_mut().position += x.len();
                self.catch_key = false;
                self.redraw();
            },
            Some(Input::Character(c)) if self.navigate_buffer.is_some() => {
                match (c, self.navigate_buffer.take()) {
                    // alt+j starts a navigation > 10
                    ('j', Some(None)) => self.navigate_buffer = Some(Some(String::new())),
                    // alt+j we are currently in alt+j mode
                    (c, Some(Some(mut buf))) => {
                        if c < '0' || c > '9' {
                            return None;
                        }
                        buf.push(c);
                        if buf.len() == 2 {
                            self.try_navigate(buf);
                        } else {
                            self.navigate_buffer = Some(Some(buf));
                        }
                    },
                    // alt+0..9
                    (c, Some(None)) if c >= '0' && c <= '9' => {
                        self.try_navigate(format!("{}", c));
                    },
                    _ => (),
                }
            },
            // Enter
            Some(Input::Character('\n')) => {
                if self.input().is_empty() {
                    return None;
                }

                let line = self.input_mut().drain(..).collect();
                self.cur_buf_mut().position = 0;
                return Some(line);
            },
            // Backspace
            Some(Input::Character('\x7f')) | Some(Input::Character('\x08')) => {
                if self.cur_buf().position > 0 {
                    self.win.mv(self.max_y -1, self.cursor_pos()-1);
                    self.win.delch();
                    self.cur_buf_mut().position -= 1;
                    let position = self.cur_buf().position;
                    self.input_mut().remove(position);
                }
            },
            // ^K
            Some(Input::Character('\x0b')) => {
                self.catch_key = true;
            },
            Some(Input::Character('\x1b')) => {
                self.navigate_buffer = Some(None);
            },
            // ^L
            Some(Input::Character('\x0c')) => {
                self.redraw();
            },
            // ^A
            Some(Input::Character('\x01')) => {
                self.cur_buf_mut().position = 0;
                self.redraw();
            },
            // ^E
            Some(Input::Character('\x05')) => {
                self.cur_buf_mut().position = self.input().len();
                self.redraw();
            },
            // ^U
            Some(Input::Character('\x15')) => {
                let position = self.cur_buf().position;
                self.input_mut().drain(..position).for_each(drop);
                self.cur_buf_mut().position = 0;
                self.redraw();
            },
            // Delete
            Some(Input::KeyDC) => {
                if self.cur_buf().position < self.input().len() {
                    self.win.delch();
                    let position = self.cur_buf().position;
                    self.input_mut().remove(position);
                }
            },
            Some(Input::Character(c)) => {
                self.win.addch(c);
                self.cur_buf_mut().position += 1;
                self.input_mut().push(c);
            },
            Some(Input::KeyLeft) => {
                self.cur_buf_mut().position = max(0, self.cur_buf().position -1);
                self.win.mv(self.max_y -1, self.cursor_pos());
            },
            Some(Input::KeyRight) => {
                self.cur_buf_mut().position = min(self.input().len(), self.cur_buf().position +1);
                self.win.mv(self.max_y -1, self.cursor_pos());
            },
            Some(Input::KeyResize) => self.resize(),
            Some(_input) => {
                // TODO: KeyPPage
                // TODO: KeyNPage
                // TODO: KeyUp
                // TODO: KeyDown
                // self.win.addstr(&format!("<{:?}>", _input));
            },
            None => (),
        }

        self.win.refresh();

        None
    }

    pub fn set_prompt<I: Into<Cow<'static, str>>>(&mut self, prompt: I) {
        self.cur_buf_mut().prompt = prompt.into();
    }

    pub fn set_topic<I: Into<Cow<'static, str>>>(&mut self, topic: I) {
        self.cur_buf_mut().topic = topic.into();
    }
}

impl Deref for Window {
    type Target = pancurses::Window;

    fn deref(&self) -> &Self::Target {
        &self.win
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        endwin();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
