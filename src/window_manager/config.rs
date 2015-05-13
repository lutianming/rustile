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

use super::handler::{ KeyBind, Handler, ExecHandler };


pub struct Config {
    mod_key: u32,
    pub bindsyms: HashMap<KeyBind, Vec<String>>,
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

impl Config {
    pub fn load() -> Config {
        let mut config = Config::default();

        let home = match env::var_os("HOME") {
            Some(v) => v,
            None => {
                // can't find HOME, return default config
                return config;
            }
        };

        let mut pathbuf = PathBuf::from(home);
        pathbuf.push("config");
        match File::open(pathbuf.as_path()) {
            Ok(f) => {
                let buf = BufReader::new(f);
                for line in buf.lines() {
                    match line {
                        Ok(s) => {
                            config.read_line(s);
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
        config
    }

    pub fn default() -> Config {
        let mut config = Config {
            mod_key: xlib::Mod4Mask,
            bindsyms: HashMap::new(),
        };

        let b = vec!["$mod+c", "exec", "dmenu_run"];
        config.bind_sym(&b);
        config
    }

    fn read_line(&self, line: String) {
        // # is comment
        if !line.starts_with("#") {
            let tokens: Vec<&str> = line.split(' ').collect();
            let (cmd, args) = tokens.split_at(1);
            match cmd[0] {
                "set" => {

                }
                "exec" => {
                    println!("{}", "exec");
                    let mut handler = ExecHandler::build(args);
                    handler.handle();
                    // self.run_cmd(&(args.to_vec()));
                }
                "bind" => {

                }
                _ => {
                    // not supported cmd, ignore
                }
            }
        }
    }

    fn bind_sym(&mut self, args: &Vec<&str>) {
        let (keyseq, cmd) = args.split_at(1);
        let keys: Vec<&str> = keyseq[0].split("+").collect();

        let bind = KeyBind::build(self.mod_key, &keys);
        println!("binding {} {}", bind.key, bind.mask);
        let mut args: Vec<String> = Vec::new();
        for c in cmd {
            args.push(c.to_string());
        }
        self.bindsyms.insert(bind, args);
    }
}
