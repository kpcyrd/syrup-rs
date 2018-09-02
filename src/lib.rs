extern crate pancurses;

use pancurses::{initscr, endwin, Input, Attribute};
use pancurses::{COLOR_PAIR, COLOR_WHITE, COLOR_BLUE};
use std::cmp::{max, min};
use std::ops::Deref;
use std::fmt::Display;

pub type Message = String;
const INPUT_HEIGHT: usize = 2; // the number of lines we need for the input area


pub struct Window {
    win: pancurses::Window,
    backlog: Vec<String>,
    input: Vec<char>,
    position: usize,
    max_y: i32,
    max_x: i32,
    prompt: String,
    /// catch the next key and add a debug representation to input
    catch_key: bool,
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
            win: win,
            backlog: Vec::new(),
            input: Vec::new(),
            position: 0,
            max_y,
            max_x,
            prompt: String::new(),
            catch_key: false,
        }
    }

    pub fn writeln<D: Display>(&mut self, txt: D) {
        let line = format!("{}\n", txt);
        self.backlog.push(line);

        if self.backlog.len() + INPUT_HEIGHT > self.max_y as usize {
            self.backlog.remove(0);
        }

        self.redraw();
    }

    pub fn redraw(&self) {
        self.win.erase();

        for line in &self.backlog {
            self.win.printw(&line);
        }

        self.draw_input();
        self.win.refresh();
    }

    pub fn draw_input(&self) {
        self.win.mv(self.max_y -2, 0);

        self.win.attrset(COLOR_PAIR(1));
        self.win.hline(' ', self.max_x);
        self.win.attrset(Attribute::Normal);

        self.win.mv(self.max_y -1, 0);
        self.win.addstr(&self.prompt);
        for x in &self.input {
            self.win.addch(*x);
        }
        self.win.mv(self.max_y -1, self.cursor_pos());
    }

    pub fn resize(&mut self) {
        let (max_y, max_x) = self.win.get_max_yx();
        self.max_y = max_y;
        self.max_x = max_x;
        while self.backlog.len() + INPUT_HEIGHT > self.max_y as usize {
            self.backlog.remove(0);
        }
        self.redraw();
    }

    fn cursor_pos(&self) -> i32 {
        (self.position + self.prompt.len()) as i32
    }

    pub fn get(&mut self) -> Option<String> {
        match self.win.getch() {
            Some(c) if self.catch_key => {
                let x = format!("{:?}", c);
                self.input.extend(x.chars());
                self.position += x.len();
                self.catch_key = false;
                self.redraw();
            },
            // Enter
            Some(Input::Character('\n')) => {
                if self.input.len() == 0 {
                    return None;
                }

                let line = self.input.drain(..).collect();
                self.position = 0;
                return Some(line);
            },
            // Backspace
            Some(Input::Character('\x7f')) | Some(Input::Character('\x08')) => {
                if self.position > 0 {
                    self.win.mv(self.max_y -1, self.cursor_pos()-1);
                    self.win.delch();
                    self.position -= 1;
                    self.input.remove(self.position);
                }
            },
            // ^K
            Some(Input::Character('\x0b')) => {
                self.catch_key = true;
            },
            // ^L
            Some(Input::Character('\x0c')) => {
                self.redraw();
            },
            // ^A
            Some(Input::Character('\x01')) => {
                self.position = 0;
                self.redraw();
            },
            // ^E
            Some(Input::Character('\x05')) => {
                self.position = self.input.len();
                self.redraw();
            },
            // ^U
            Some(Input::Character('\x15')) => {
                self.input.drain(..self.position).for_each(drop);
                self.position = 0;
                self.redraw();
            },
            // Delete
            Some(Input::KeyDC) => {
                if self.position < self.input.len() {
                    self.win.delch();
                    self.input.remove(self.position);
                }
            },
            Some(Input::Character(c)) => {
                self.win.addch(c);
                self.position += 1;
                self.input.push(c);
            },
            Some(Input::KeyLeft) => {
                self.position = max(0, self.position -1);
                self.win.mv(self.max_y -1, self.cursor_pos());
            },
            Some(Input::KeyRight) => {
                self.position = min(self.input.len(), self.position +1);
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

    pub fn set_prompt<I: Into<String>>(&mut self, prompt: I) {
        self.prompt = prompt.into();
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
