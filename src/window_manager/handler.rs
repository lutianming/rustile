extern crate x11;
extern crate libc;

use std::ptr;
use std::process::Command;
use std::boxed::Box;

use x11::xlib;
use x11::xlib::{ Display, Window };
use super::WindowManager;
use super::workspace::Workspaces;
use super::layout;
use super::super::libx;
use super::super::libx::Context;

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
                "Mod1" => mask = mask | xlib::Mod1Mask,
                "Mod2" => mask = mask | xlib::Mod2Mask,
                "Mod3" => mask = mask | xlib::Mod3Mask,
                "Mod4" => mask = mask | xlib::Mod4Mask,
                "Mod5" => mask = mask | xlib::Mod5Mask,
                _ => {
                    sym = libx::string_to_keysym(key);
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
    fn handle(&mut self, workspaces: &mut Workspaces, context: Context, screen_num: libc::c_int);
}

pub struct ExecHandler {
    pub cmd: Command
}

pub struct LayoutHandler {
    layout_type: layout::Type,
}

/// switch to another workspace
pub struct WorkspaceHandler {
    pub key: char,
}

/// move window to target workspace
pub struct WindowToWorkspaceHandler {
    pub key: char,
}

pub struct WindowFocusHandler {
    pub direction: layout::Direction,
}

pub struct WindowCloseHandler;

impl Handler for WindowCloseHandler {
    fn handle(&mut self, workspaces: &mut Workspaces, context: Context, screen_num: libc::c_int) {
        debug!("handle window close");
        let (window, _) = libx::get_input_focus(context);
        libx::kill_window(context, window);
        // xlib::XWithdrawWindow(context, window, screen_num);
        // xlib::XDestroyWindow(context, window);
        println!("try kill window {}", window);

    }
}
impl Handler for WindowFocusHandler {
    fn handle(&mut self, workspaces: &mut Workspaces, context: Context, screen_num: libc::c_int) {
        debug!("change focus in current workspace");
        let (window, _) = libx::get_input_focus(context);

        println!("window {}", window);
        let current = workspaces.current();
        match current.contain(window) {
            Some(i) => {
                let next = if (i+1) >= current.size() { 0 } else { i+1 };
                libx::set_input_focus(context, current.get(next));
            }
            None => {}
        }

    }
}

impl Handler for WindowToWorkspaceHandler {
    fn handle(&mut self, workspaces: &mut Workspaces, context: Context, screen_num: libc::c_int) {
        debug!("handle window move form {} to {}", workspaces.current_name(), self.key);

        let (window, _) = libx::get_input_focus(context);
        println!("window {}", window);

        // let mut root: Window = 0;
        // let mut parent: Window = window;
        // let mut children: *mut Window = ptr::null_mut();
        // let mut nchildren: libc::c_uint = 0;

        // while parent != root {
        //     window = parent;
        //     unsafe{
        //         let s = xlib::XQueryTree(context, window, &mut root, &mut parent, &mut children, &mut nchildren);
        //         if s > 0 {
        //             println!("parent {} root {}", parent, root);
        //             xlib::XFree(children as *mut libc::c_void);
        //         }
        //         else{
        //             println!("error");
        //         }
        //     }
        // }

        println!("focused window {}", window);
        let from = workspaces.current_name();
        let to = self.key;

        if !workspaces.contain(to) {
            workspaces.create(to, screen_num);
        }

        workspaces.move_window(window, from, to);
        match workspaces.get(from){
            Some(w) => { w.config(context)}
            None => {}
        };

        match workspaces.get(self.key) {
            Some(w) => {
                w.config(context)
            }
            None => {}
        };
    }
}

impl Handler for WorkspaceHandler {
    fn handle(&mut self, workspaces: &mut Workspaces, context: Context, screen_num: libc::c_int) {
        debug!("handle workspace form {}", workspaces.current_name());
        if !workspaces.contain(self.key) {
            workspaces.create(self.key, screen_num);
        }
        workspaces.switch_current(self.key, context);
        println!("to {}", workspaces.current_name());
    }
}

impl Handler for ExecHandler {
    fn handle(&mut self, workspaces: &mut Workspaces, context: Context, screen_num: libc::c_int) {
        debug!("handle exec");
        self.cmd.spawn();
    }
}

impl Handler for LayoutHandler {
    fn handle(&mut self, workspaces: &mut Workspaces, context: Context, screen_num: libc::c_int) {
        debug!("handle layout");
        let current = workspaces.current();
        let t = self.layout_type.clone();
        current.change_layout(t);
        current.config(context);
    }
}

impl LayoutHandler {
    pub fn new(layout: layout::Type) -> LayoutHandler{
        LayoutHandler {
            layout_type: layout
        }
    }
}

impl ExecHandler {
    pub fn new(tokens: &[&str]) -> ExecHandler {
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
