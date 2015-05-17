extern crate x11;
extern crate libc;

use std::ffi;
use std::mem;
use std::ptr;
use std::process::Command;
use std::boxed::Box;

use x11::xlib;
use x11::xlib::{ Display, Window };
use super::WindowManager;
use super::workspace::Workspaces;
use super::layout;
use super::super::libx;

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
    fn handle(&mut self, workspaces: &mut Workspaces, display: *mut Display, screen_num: libc::c_int);
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
    fn handle(&mut self, workspaces: &mut Workspaces, display: *mut Display, screen_num: libc::c_int) {
        debug!("handle window close");
        let (window, _) = libx::get_input_focus(display);
        let mut event: xlib::XClientMessageEvent = unsafe {
            mem::zeroed()
        };
        let wm_delete_window = libx::get_atom(display, "WM_DELETE_WINDOW");
        let wm_protocols = libx::get_atom(display, "WM_PROTOCOLS");

        println!("wm delete window {}", wm_delete_window);
        println!("protocols {}", wm_protocols);

        event.type_ = xlib::ClientMessage;
        event.message_type = wm_protocols;
        event.format = 32;
        event.window = window;
        event.send_event = xlib::True;
        event.display = display;
        event.data.set_long(0, wm_delete_window as libc::c_long);

        let mut e: xlib::XEvent = From::from(event);
        libx::send_event(display, window, xlib::True, xlib::NoEventMask, e);

        // xlib::XWithdrawWindow(display, window, screen_num);
        // xlib::XDestroyWindow(display, window);
        println!("destroy {}", window);

    }
}
impl Handler for WindowFocusHandler {
    fn handle(&mut self, workspaces: &mut Workspaces, display: *mut Display, screen_num: libc::c_int) {
        debug!("change focus in current workspace");
        let (window, _) = libx::get_input_focus(display);

        println!("window {}", window);
        let current = workspaces.current();
        match current.contain(window) {
            Some(i) => {
                let next = if (i+1) >= current.size() { 0 } else { i+1 };
                libx::set_input_focus(display, current.get(next), 0, 0);
            }
            None => {}
        }

    }
}

impl Handler for WindowToWorkspaceHandler {
    fn handle(&mut self, workspaces: &mut Workspaces, display: *mut Display, screen_num: libc::c_int) {
        debug!("handle window move form {} to {}", workspaces.current_name(), self.key);

        let (window, _) = libx::get_input_focus(display);
        println!("window {}", window);

        let mut root: Window = 0;
        let mut parent: Window = window;
        let mut children: *mut Window = ptr::null_mut();
        let mut nchildren: libc::c_uint = 0;

        // while parent != root {
        //     window = parent;
        //     unsafe{
        //         let s = xlib::XQueryTree(display, window, &mut root, &mut parent, &mut children, &mut nchildren);
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
        workspaces.current().p();
        let from = workspaces.current_name();
        workspaces.move_window(window, from, self.key);
        workspaces.current().config(display, screen_num);

        match workspaces.get(self.key) {
            Some(w) => {
                w.config(display, screen_num);
            }
            None => {}
        }


    }
}

impl Handler for WorkspaceHandler {
    fn handle(&mut self, workspaces: &mut Workspaces, display: *mut Display, screen_num: libc::c_int) {
        debug!("handle workspace");
        workspaces.switch_current(self.key, display);

    }
}

impl Handler for ExecHandler {
    fn handle(&mut self, workspaces: &mut Workspaces, display: *mut Display, screen_num: libc::c_int) {
        debug!("handle exec");
        self.cmd.spawn();
    }
}

impl Handler for LayoutHandler {
    fn handle(&mut self, workspaces: &mut Workspaces, display: *mut Display, screen_num: libc::c_int) {
        debug!("handle layout");
        let current = workspaces.current();
        let t = self.layout_type.clone();
        current.change_layout(t);
        current.config(display, screen_num);
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
