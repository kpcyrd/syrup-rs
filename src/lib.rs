extern crate pancurses;
extern crate textwrap;

use pancurses::{initscr, endwin, Input, Attribute};
use pancurses::{COLOR_PAIR, COLOR_WHITE, COLOR_BLUE};
use std::borrow::Cow;
use std::cmp::min;
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
    pub fn new() -> Buffer {
        Buffer {
            backlog: Vec::new(),
            input: Vec::new(),
            position: 0,
            prompt: "".into(),
            topic: "".into(),
        }
    }

    fn cursor_pos(&self) -> i32 {
        (self.position + self.prompt.len()) as i32
    }

    pub fn get(&mut self, win: &mut pancurses::Window, key: Input, max_y: i32) -> (bool, Option<String>) {
        match key {
            // Enter
            Input::Character('\n') => {
                if self.input.is_empty() {
                    return (false, None);
                }

                let line = self.input.drain(..).collect();
                self.position = 0;
                return (false, Some(line));
            },
            // Backspace
            Input::Character('\x7f') | Input::Character('\x08') => {
                if self.position > 0 {
                    win.mv(max_y -1, self.cursor_pos()-1);
                    win.delch();
                    self.position -= 1;
                    self.input.remove(self.position);
                }
            },
            // Delete
            Input::KeyDC => {
                if self.position < self.input.len() {
                    win.delch();
                    self.input.remove(self.position);
                }
            },
            // ^A
            Input::Character('\x01') => {
                self.position = 0;
                // redraw
                return (true, None);
            },
            // ^E
            Input::Character('\x05') => {
                self.position = self.input.len();
                // redraw
                return (true, None);
            },
            // ^U
            Input::Character('\x15') => {
                self.input.drain(..self.position).for_each(drop);
                self.position = 0;
                // redraw
                return (true, None);
            },
            Input::Character(c) => {
                win.addch(c);
                self.position += 1;
                self.input.push(c);
            },
            Input::KeyLeft => {
                if self.position > 0 {
                    self.position -= 1;
                    win.mv(max_y -1, self.cursor_pos());
                }
            },
            Input::KeyRight => {
                self.position = min(self.input.len(), self.position +1);
                win.mv(max_y -1, self.cursor_pos());
            },
            _input => {
                // TODO: KeyPPage
                // TODO: KeyNPage
                // TODO: KeyUp
                // TODO: KeyDown
                // self.win.addstr(&format!("<{:?}>", _input));
            },
        }

        win.refresh();

        (false, None)
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
        for x in &self.cur_buf().input {
            self.win.addch(*x);
        }
        self.win.mv(self.max_y -1, self.cur_buf().cursor_pos());
    }

    pub fn resize(&mut self) {
        let (max_y, max_x) = self.win.get_max_yx();
        self.max_y = max_y;
        self.max_x = max_x;
        self.redraw();
    }

    pub fn cur_buf(&self) -> &Buffer {
        self.buffers.get(self.cur_buf).unwrap()
    }

    pub fn cur_buf_mut(&mut self) -> &mut Buffer {
        self.buffers.get_mut(self.cur_buf).unwrap()
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
            Some(Input::KeyResize) => self.resize(),
            Some(input) => {
                let (redraw, result) = {
                    let buffers = &mut self.buffers;
                    let win = &mut self.win;
                    buffers.get_mut(self.cur_buf).unwrap().get(win, input, self.max_y)
                };
                if redraw {
                    self.redraw();
                }
                return result;
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
