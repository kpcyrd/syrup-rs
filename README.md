# syrup-rs [![Build Status][travis-img]][travis] [![crates.io][crates-img]][crates] [![docs.rs][docs-img]][docs]

[travis-img]:   https://travis-ci.org/kpcyrd/syrup-rs.svg?branch=master
[travis]:       https://travis-ci.org/kpcyrd/syrup-rs
[crates-img]:   https://img.shields.io/crates/v/syrup.svg
[crates]:       https://crates.io/crates/syrup
[docs-img]:     https://docs.rs/syrup/badge.svg
[docs]:         https://docs.rs/syrup

Simple abstraction around pancurses for chat-like interfaces.

```
# Cargo.toml
[dependencies]
syrup = "0.1"
```

To get started, see the [docs] and the examples/ folder.
```rust
extern crate syrup;

use syrup::Window;

use std::thread;
use std::sync::mpsc;
use std::time::Duration;


fn main() {
    let mut window = Window::initscr();
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
            }

            window.writeln(format!("< {:?}", line));
        }
    }
}
```

## License

MIT/Apache-2.0
