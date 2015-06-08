extern crate x11;

use std::collections::HashMap;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Lines;
use std::fs::File;
use std::path::PathBuf;
use std::process::Command;
use std::ffi;
use std::env;
use x11::xlib;
use std::boxed::Box;

use super::layout;
use super::handler::{self, KeyBind};

pub struct Config {
    mod_key: u32,
    pub bindsyms: HashMap<KeyBind, Box<handler::Handler>>,
    pub titlebar_height: u32,
}

impl Config {
    pub fn new() -> Config {
        Config {
            mod_key: xlib::Mod4Mask,
            bindsyms: HashMap::new(),
            titlebar_height: 16,
        }
    }

    pub fn load(&mut self) {
        let mut pathbuf = match env::var_os("HOME") {
            Some(v) => {
                PathBuf::from(v)
            }
            None => {
                // can't find HOME, do nothing
                return;
            }
        };

        pathbuf.push(".rustile");
        match File::open(pathbuf.as_path()) {
            Ok(f) => {
                let buf = BufReader::new(f);
                for line in buf.lines() {
                    match line {
                        Ok(s) => {
                            self.parse_line(s);
                        }
                        Err(err) => {
                            // do nothing
                        }
                    }
                }
            }
            Err(err) => {
                // use default config
                println!("no config file");
            }
        }
    }

    fn parse_line(&mut self, line: String) {
        // # is comment
        if !line.starts_with("#") {
            let tokens: Vec<&str> = line.split(' ').collect();
            let (cmd, args) = tokens.split_at(1);
            match cmd[0] {
                "set" => {
                    if args.len() > 1 {
                        self.set_var(args[0], args[1]);
                    }
                }
                "exec" => {
                    let mut handler = handler::ExecHandler::new(args);
                    handler.cmd.spawn();
                }
                "bind" => {
                    self.bind_sym(args);
                }
                _ => {
                    // not supported cmd, ignore
                }
            }
        }
    }

    fn bind_sym(&mut self, args: &[&str]) {
        let (keyseq, cmd) = args.split_at(1);
        let keys: Vec<&str> = keyseq[0].split("+").collect();

        let bind = KeyBind::build(self.mod_key, &keys);

        let (name, args) = cmd.split_at(1);
        match name[0] {
            "exec" => {
                println!("exec");
                let handler = handler::ExecHandler::new(args);
                self.bindsyms.insert(bind, Box::new(handler));
            }
            "layout" => {
                println!("layout");
                let layout = args[0];
                match layout {
                    "split" => {
                        let handler = handler::LayoutHandler::new(layout::Type::Tiling);
                        self.bindsyms.insert(bind, Box::new(handler));
                    }
                    "tab" => {
                        let handler = handler::LayoutHandler::new(layout::Type::Tab);
                        self.bindsyms.insert(bind, Box::new(handler));
                    }
                    _ => {}
                }
            }
            "fullscreen" => {
                let handler = handler::FullscreenHandler;
                self.bindsyms.insert(bind, Box::new(handler));
            }
            "split" => {
                let handler = handler::SplitHandler;
                self.bindsyms.insert(bind, Box::new(handler));
            }
            "workspace" => {
                println!("workspace");
                let c = args[0].chars().nth(0);
                match c {
                    Some(v) => {
                        let handler = handler::WorkspaceHandler {
                            key: v,
                        };
                        self.bindsyms.insert(bind, Box::new(handler));
                    }
                    None => {}
                }
            }
            "window" => {
                let c = args[0].chars().nth(0);
                match c {
                    Some(v) => {
                        println!("window");
                        let handler = handler::WindowToWorkspaceHandler {
                            key: v,
                        };
                        self.bindsyms.insert(bind, Box::new(handler));
                    }
                    None => {}
                }
            }
            "resize" => {
                let resize = match args[0] {
                    "shrink" => {
                        handler::Resize::Shrink
                    }
                    "grow" => {
                        handler::Resize::Grow
                    }
                    _ => { handler::Resize::Grow }
                };
                let direction = match args[1] {
                    "width" => {
                        layout::Direction::Vertical
                    }
                    "height" => {
                        layout::Direction::Horizontal
                    }
                    _ => { layout::Direction::Horizontal }
                };
                let handler = handler::WindowResizeHandler {
                    direction: direction,
                    resize: resize
                };
                self.bindsyms.insert(bind, Box::new(handler));
            }
            "focus" => {
                let direction = match args[0] {
                    "left" => layout::Direction::Left,
                    "right" => layout::Direction::Right,
                    "up" => layout::Direction::Up,
                    "down" => layout::Direction::Down,
                    _ => layout::Direction::Right
                };
                let handler = handler::WindowFocusHandler {
                    direction: direction
                };
                self.bindsyms.insert(bind, Box::new(handler));
            }
            "kill" => {
                let handler = handler::WindowCloseHandler;
                self.bindsyms.insert(bind, Box::new(handler));
            }
            _ => {}
        };
    }

    fn set_var(&mut self, key: &str, val: &str) {
        match key {
            "$mod" => {
                self.mod_key = match val {
                    "Shift" => xlib::ShiftMask,
                    "Ctrl"  => xlib::ControlMask,
                    "Mod1"  => xlib::Mod1Mask,
                    "Mod2" => xlib::Mod2Mask,
                    "Mod3" => xlib::Mod3Mask,
                    "Mod4" => xlib::Mod4Mask,
                    "Mod5" => xlib::Mod5Mask,
                    _ => xlib::Mod4Mask
                };
            }
            _ => {}
        }
    }
}


#[test]
fn test_map() {
    let mut bindsyms: HashMap<KeyBind, i32> = HashMap::new();
    let b = KeyBind {
        key: 0,
        mask: 0
    };
    bindsyms.insert(b, 1);

    let c = KeyBind {
        key: 0,
        mask: 0
    };
    assert!(bindsyms.contains_key(&c), true);
}
