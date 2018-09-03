extern crate syrup;

use syrup::Window;

use std::thread;
use std::sync::mpsc;
use std::time::Duration;


fn main() {
    let mut window = Window::initscr();

    window.set_topic("syrup example/chat.rs topic");
    window.set_prompt("[user] ");

    window.writeln("");
    window.writeln(" === welcome to example chat");
    window.writeln("");

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        loop {
            tx.send(String::from("ohai")).unwrap();
            thread::sleep(Duration::from_secs(3));
        }
    });

    loop {
        if let Ok(msg) = rx.try_recv() {
            window.writeln(format!("> {:?}", msg));
        }

        if let Some(line) = window.get() {
            if line == "/quit" {
                break;
            } else if line.starts_with("/topic ") {
                window.set_topic(line[7..].to_string());
                window.redraw();
            } else {
                window.writeln(format!("< {:?}", line));
            }
        }
    }
}
