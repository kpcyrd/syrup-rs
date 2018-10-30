extern crate syrup;

use syrup::Window;

use std::thread;
use std::sync::mpsc;
use std::time::Duration;

const IPSUM: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Phasellus vel sapien vitae quam facilisis convallis volutpat et elit. Cras quis justo finibus, rutrum justo elementum, volutpat quam. Vestibulum commodo urna lobortis, bibendum arcu eu, maximus urna. Aenean augue tellus, molestie ut augue a, feugiat faucibus ligula. Fusce mattis luctus lacus, eu euismod neque placerat sed. Morbi vel eleifend velit, vel commodo purus. Mauris quis tincidunt nunc, eu finibus nisi. Nunc consequat, velit sed aliquet luctus, ex purus tristique turpis, eget venenatis turpis felis a purus. Nam finibus lectus in quam rutrum, at feugiat magna venenatis. Donec imperdiet gravida lectus sed cursus. Proin volutpat ligula vel quam efficitur fringilla. Donec hendrerit urna ut ultricies dapibus.";


fn main() {
    let mut window = Window::initscr();

    window.set_topic("🤷".repeat(300));
    window.set_prompt("[user] ");

    window.writeln("");
    window.writeln(" === welcome to example chat");
    window.writeln("");

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let emojis = "🤷".repeat(300);
        let tabs = "\t".repeat(30);

        loop {
            // include format strings after this was fixed in pancurses
            for x in &[IPSUM, &emojis, &tabs] {
                tx.send(x.to_string()).unwrap();
                thread::sleep(Duration::from_secs(3));
            }
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
