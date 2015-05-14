extern crate x11;

use std::ffi;
use std::process::Command;
use x11::xlib;

#[derive(Hash, Eq, PartialEq, Debug)]
pub struct KeyBind {
    pub key: u64,
    pub mask: u32,
}

impl KeyBind {
    pub fn build(mod_key: u32, tokens: &[&str]) -> KeyBind {
        let mut mask = 0;
        let mut sym = 0;
        for key in tokens {
            match *key {
                "$mod" => mask = mask | mod_key,
                "Shift" => mask = mask | xlib::ShiftMask,
                "Ctrl" => mask = mask | xlib::ControlMask,
                "mod1" => mask = mask | xlib::Mod1Mask,
                "mod2" => mask = mask | xlib::Mod2Mask,
                "mod3" => mask = mask | xlib::Mod3Mask,
                "mod4" => mask = mask | xlib::Mod4Mask,
                "mod5" => mask = mask | xlib::Mod5Mask,
                _ => {
                    let tmp = ffi::CString::new(*key).unwrap();
                    unsafe{
                        sym = xlib::XStringToKeysym(tmp.as_ptr());
                    }
                }
            }
        }

        println!("bind {} {}", mask, sym);
        KeyBind {
            mask: mask,
            key: sym
        }
    }
}
pub trait Handler {
    fn handle(&mut self);
}

pub struct ExecHandler {
    cmd: Command
}

impl Handler for ExecHandler {
    fn handle(&mut self) {
        self.cmd.spawn();
    }
}
impl ExecHandler {
    pub fn build(tokens: &[&str]) -> ExecHandler {
        let (name, args) = tokens.split_at(1);
        let mut cmd = Command::new(name[0]);

        for arg in args {
            println!("{}", arg);
            cmd.arg(arg);
        }

        let handler = ExecHandler {
            cmd: cmd,
        };
        handler
    }
}
