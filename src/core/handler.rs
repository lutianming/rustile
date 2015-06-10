extern crate x11;
extern crate libc;

use std::ptr;
use std::process::Command;
use std::boxed::Box;

use x11::xlib;
use super::WindowManager;
use super::Workspaces;
use super::container;
use super::layout::{self, LayoutDirection, MoveDirection};
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

pub enum Resize {
    Shrink,
    Grow,
}

pub type HandleFn = Box<FnMut(&mut Workspaces)>;

pub fn exec(mut cmd: Command) -> HandleFn {
    Box::new(move |workspaces| {cmd.spawn();})
}

pub fn switch_workspace(key: char) -> HandleFn {
    let f = move |workspaces: &mut Workspaces| {
        workspaces.switch_workspace(key);
    };
    Box::new(f)
}

pub fn layout(layout: layout::Type) -> HandleFn {
    let f = move |workspaces: &mut Workspaces| {
        let old = workspaces.mode;
        workspaces.mode = container::Mode::Layout;
        let mut changed = false;
        if let Some(container) = workspaces.get_focus() {
            let c = if container.get_parent().is_some() {
                container.get_parent().unwrap()
            }
            else {
                container
            };
            c.change_layout(layout.clone());
            c.update_layout();
            changed = true;
        }
        if !changed {
            workspaces.mode = old;
        }
    };
    Box::new(f)
}

pub fn fullscreen() -> HandleFn {
    Box::new(move |workspaces| {
        if let Some(c) = workspaces.get_focus(){
            c.mode_toggle();
        }
    })
}

pub fn move_window_to_workspace(key: char) -> HandleFn {
    Box::new(move |workspaces| {
        let id = match workspaces.get_focus() {
            Some(container) => {
                container.raw_id()
            }
            None => { return }
        };
        let from = workspaces.current_name();
        let to = key;
        workspaces.move_window(id, from, to);
    })
}

pub fn focus_window(direction: MoveDirection) -> HandleFn {
    Box::new(move |workspaces| {
        if let Some(c) = workspaces.get_focus() {
            if let Some(p) = c.get_parent() {
                let index = p.contain(c.raw_id()).unwrap();

                if let Some(next) = p.circulate(index, direction.clone()) {
                    if next != index {
                        c.unfocus();
                        p.get_child(next).unwrap().focus();
                    }
                }
            }
        }
    })
}

pub fn close_window() -> HandleFn {
    Box::new(move |workspaces| {
        let context = workspaces.context;
        if let Some(c) = workspaces.get_focus() {
            libx::kill_window(context, c.raw_id());
            println!("try kill window {}", c.raw_id());
        }
    })
}

pub fn split_container() -> HandleFn {
    Box::new(move |workspaces| {
        workspaces.current().print_tree(0);
        if let Some(c) = workspaces.get_focus() {
            c.split();
        }
        workspaces.current().print_tree(0);
    })
}

pub fn resize_window(direction: LayoutDirection, resize: Resize) -> HandleFn {
    Box::new(move |workspaces| {
        let old = workspaces.mode;
        workspaces.mode = container::Mode::Layout;
        let mut changed = false;
        if let Some(c) = workspaces.get_focus() {
            if let Some(p) = c.get_parent() {
                let index = p.contain(c.raw_id()).unwrap();
                if p.direction == direction {
                    let step:f32 = match resize {
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
                    changed = true;
                }
            }
        }
        if !changed {
            workspaces.mode = old;
        }
    })
}
