extern crate x11;
extern crate libc;

use std::ptr;
use std::process::Command;
use std::boxed::Box;

use x11::xlib;
use super::WindowManager;
use super::Workspaces;
use super::container;
use super::layout::{self, Direction};
use super::super::libx::{self, Context};

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

pub enum Resize {
    Shrink,
    Grow,
}

pub struct WindowResizeHandler {
    pub direction: layout::Direction,
    pub resize: Resize
}

impl Handler for WindowResizeHandler {
    fn handle(&mut self, workspaces: &mut Workspaces, context: Context) {
        let focused = workspaces.get_focus();
        if focused.is_some() {
            let c = focused.unwrap();
            let parent = c.get_parent();
            if parent.is_some() {
                let p = parent.unwrap();
                let index = p.contain(c.raw_id()).unwrap();
                if p.direction == self.direction {
                    let step:f32 = match self.resize {
                        Resize::Shrink => {
                            -0.05
                        }
                        Resize::Grow => {
                            0.05
                        }
                    };

                    if index > 0 && index < (p.size() - 1) {
                        p.resize_children(index, index-1, step);
                        p.resize_children(index, index+1, step);
                    }
                    else if index > 0 {
                        p.resize_children(index, index-1, step*2.0);
                    }
                    else if index < (p.size() - 1) {
                        p.resize_children(index, index+1, step*2.0);
                    }
                    p.update_layout();
                }

            }
        }
    }
}

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
                container.raw_id()
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
        let t = self.layout_type.clone();
        let focused = workspaces.get_focus();
        match focused {
            Some(container) => {
                if container.get_parent().is_some() {
                    let p = container.get_parent().unwrap();
                    p.change_layout(t);
                    p.update_layout();
                }
                else {
                    container.change_layout(t);
                    container.update_layout();
                }
            }
            None => {}
        }
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
