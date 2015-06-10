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

use super::layout::{self, LayoutDirection, MoveDirection};
use super::handler::{self, KeyBind};

pub fn build_cmd(tokens: &[&str]) -> Command {
    let (name, args) = tokens.split_at(1);
    let mut cmd = Command::new(name[0]);
    cmd.args(args);
    cmd
}

pub struct Config {
    mod_key: u32,
    pub bindsyms: HashMap<KeyBind, handler::HandleFn>,
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
                    let mut cmd = build_cmd(&tokens);
                    cmd.spawn();
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
                let mut cmd = build_cmd(args);
                let handler = handler::exec(cmd);
                self.bindsyms.insert(bind, handler);
            }
            "layout" => {
                println!("layout");
                let layout = args[0];
                match layout {
                    "split" => {
                        let handler = handler::layout(layout::Type::Tiling);
                        self.bindsyms.insert(bind, handler);
                    }
                    "tab" => {
                        let handler = handler::layout(layout::Type::Tab);
                        self.bindsyms.insert(bind, handler);
                    }
                    _ => {}
                }
            }
            "fullscreen" => {
                let handler = handler::fullscreen();
                self.bindsyms.insert(bind, handler);
            }
            "split" => {
                let handler = handler::split_container();
                self.bindsyms.insert(bind, handler);
            }
            "workspace" => {
                println!("workspace");
                let c = args[0].chars().nth(0);
                match c {
                    Some(v) => {
                        let handler = handler::switch_workspace(v);
                        self.bindsyms.insert(bind, handler);
                    }
                    None => {}
                }
            }
            "window" => {
                let c = args[0].chars().nth(0);
                match c {
                    Some(v) => {
                        println!("window");
                        let handler = handler::move_window_to_workspace(v);
                        self.bindsyms.insert(bind, handler);
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
                        LayoutDirection::Vertical
                    }
                    "height" => {
                        LayoutDirection::Horizontal
                    }
                    _ => { LayoutDirection::Horizontal }
                };
                let handler = handler::resize_window(direction, resize);
                self.bindsyms.insert(bind, handler);
            }
            "focus" => {
                let direction = match args[0] {
                    "left" => MoveDirection::Left,
                    "right" => MoveDirection::Right,
                    "up" => MoveDirection::Up,
                    "down" => MoveDirection::Down,
                    _ => MoveDirection::Right
                };
                let handler = handler::focus_window(direction);
                self.bindsyms.insert(bind, handler);
            }
            "kill" => {
                let handler = handler::close_window();
                self.bindsyms.insert(bind, handler);
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
