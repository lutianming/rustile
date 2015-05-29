extern crate x11;
extern crate libc;

use std::ptr;
use std::process::Command;
use std::boxed::Box;

use x11::xlib;
use super::WindowManager;
use super::Workspaces;
use super::container;
use super::layout;
use super::super::libx;
use super::super::libx::Context;

#[derive(Hash, Eq, PartialEq, Debug)]
pub struct KeyBind {
    pub key: xlib::KeySym,
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
    fn handle(&mut self, workspaces: &mut Workspaces, context: Context);
}

pub struct ExecHandler {
    pub cmd: Command
}

pub struct LayoutHandler {
    layout_type: layout::Type,
}

pub struct FullscreenHandler;
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
pub struct SplitHandler;

impl Handler for SplitHandler {
    fn handle(&mut self, workspaces: &mut Workspaces, context: Context) {
        workspaces.current().print_tree(0);
        match workspaces.get_focus() {
            Some(c) => {
                c.split();
            }
            None => {}
        }
        println!("after");
        workspaces.current().print_tree(0);
    }
}

impl Handler for WindowCloseHandler {
    fn handle(&mut self, workspaces: &mut Workspaces, context: Context) {
        debug!("handle window close");
        let (window, _) = libx::get_input_focus(context);
        libx::kill_window(context, window);
        // xlib::XWithdrawWindow(context, window, screen_num);
        // xlib::XDestroyWindow(context, window);
        println!("try kill window {}", window);

    }
}
impl Handler for WindowFocusHandler {
    fn handle(&mut self, workspaces: &mut Workspaces, context: Context) {
        debug!("change focus in current workspace");
        // let res = workspaces.get_focus(context);

        // match res {
        //     Some(container) => {
        //         println!("window {}", container.id);
        //         container.unfocus();

        //         let current = workspaces.current();
        //         match current.contain(container.id) {
        //             Some(i) => {
        //                 let next = current.next_client(container.id);
        //                 next.unwrap().focus();
        //             }
        //             None => {}
        //         }
        //     }
        //     None => {}
        // }
        let current = workspaces.current();
        current.switch_client();
    }
}

impl Handler for WindowToWorkspaceHandler {
    fn handle(&mut self, workspaces: &mut Workspaces, context: Context) {
        let id = match workspaces.get_focus() {
            Some(container) => {
                container.id
            }
            None => { return }
        };
        let from = workspaces.current_name();
        let to = self.key;
        workspaces.move_window(id, from, to);
    }
}

impl Handler for FullscreenHandler {
    fn handle(&mut self, workspaces: &mut Workspaces, context: Context) {
        debug!("fullscreen toggle");
        let focused = workspaces.get_focus();
        match focused {
            Some(c) => {
                c.mode_toggle();
            }
            None => {}
        }
    }
}
impl Handler for WorkspaceHandler {
    fn handle(&mut self, workspaces: &mut Workspaces, context: Context) {
        debug!("handle workspace form {}", workspaces.current_name());
        if !workspaces.contain(self.key) {
            workspaces.create(self.key);
        }
        workspaces.switch_workspace(self.key, context);
        println!("to {}", workspaces.current_name());
    }
}

impl Handler for ExecHandler {
    fn handle(&mut self, workspaces: &mut Workspaces, context: Context) {
        debug!("handle exec");
        self.cmd.spawn();
    }
}

impl Handler for LayoutHandler {
    fn handle(&mut self, workspaces: &mut Workspaces, context: Context) {
        debug!("handle layout");
        let current = workspaces.current();
        let t = self.layout_type.clone();
        current.change_layout(t);
        current.update_layout();
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
        cmd.args(args);

        let handler = ExecHandler {
            cmd: cmd,
        };
        handler
    }
}
