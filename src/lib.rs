extern crate pancurses;

use pancurses::{initscr, endwin, Input, noecho};
use std::cmp::{max, min};
use std::ops::Deref;
use std::fmt::Display;

pub type Message = String;
const INPUT_HEIGHT: usize = 2; // the number of lines we need for the input area


pub struct Window {
    win: pancurses::Window,
    backlog: Vec<String>,
    input: Vec<char>,
    position: i32,
    max_y: i32,
    max_x: i32,
}

impl Window {
    pub fn initscr() -> Window {
        let win = initscr();
        win.timeout(100);
        win.keypad(true);
        noecho();

        let (max_y, max_x) = win.get_max_yx();

        Window {
            win: win,
            backlog: Vec::new(),
            input: Vec::new(),
            position: 0,
            max_y,
            max_x,
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
        self.win.hline('-', self.max_x);
        self.win.mv(self.max_y -1, 0);
        for x in &self.input {
            self.win.addch(*x);
        }
        self.win.mv(self.max_y -1, self.position);
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

    pub fn get(&mut self) -> Option<String> {
        match self.win.getch() {
            Some(Input::Character('\n')) => {
                if self.input.len() == 0 {
                    return None;
                }

                let line = self.input.drain(..).collect();
                self.position = 0;
                return Some(line);
            },
            Some(Input::Character('\x7f')) => {
                if self.position > 0 {
                    self.win.mv(self.max_y -1, self.position-1);
                    self.win.delch();
                    self.position -= 1;
                    self.input.remove(self.position as usize);
                }
            },
            Some(Input::KeyDC) => {
                if self.position < self.input.len() as i32 {
                    self.win.delch();
                    self.input.remove(self.position as usize);
                }
            },
            Some(Input::Character(c)) => {
                self.win.addch(c);
                self.position += 1;
                self.input.push(c);
            },
            Some(Input::KeyLeft) => {
                self.position = max(0, self.position -1);
                self.win.mv(self.max_y -1, self.position);
            },
            Some(Input::KeyRight) => {
                self.position = min(self.input.len() as i32, self.position +1);
                self.win.mv(self.max_y -1, self.position);
            },
            Some(Input::KeyResize) => self.resize(),
            Some(_input) => {
                // TODO: KeyPPage
                // TODO: KeyNPage
                // TODO: KeyUp
                // TODO: KeyDown
                // self.win.addstr(&format!("<{:?}>", input));
            },
            None => (),
        }

        self.win.refresh();

        None
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