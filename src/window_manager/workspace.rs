extern crate libc;

use x11::xlib;
use x11::xlib::{ Window, Display };

use std::boxed::Box;
use std::collections::HashMap;
use super::layout;
use super::WindowManager;

pub struct Workspace {
    root: Window,
    windows: Vec<Window>,
    pub layout: Box<layout::Layout>
}

impl Workspace {
    pub fn new() -> Workspace {
        Workspace {
            root: 0,
            windows: Vec::new(),
            layout: Box::new(layout::TilingLayout::new(layout::Direction::Horizontal))
        }
    }

    pub fn add(&mut self, window: Window) {
        self.windows.push(window);
    }

    pub fn remove(&mut self, window: Window) {
        let index = self.contain(window);
        match index {
            Some(i) => { self.windows.remove(i); }
            None => {}
        };
    }

    pub fn hide(&mut self, display: *mut Display) {
        for w in self.windows.iter() {
            unsafe{
                xlib::XUnmapWindow(display, *w);
            }
        }
    }

    pub fn show(&mut self, display: *mut Display) {
        for w in self.windows.iter() {
            unsafe{
                xlib::XMapWindow(display, *w);
            }
        }
    }

    pub fn contain(&self, window: Window) -> Option<usize>{
        self.windows.iter().position(|x| *x == window)
    }

    pub fn change_layout(&mut self, layout_type: layout::Type) {
        let t = self.layout.get_type();
        if t == layout_type {
            self.layout.toggle();
        }
        else{
            match layout_type {
                layout::Type::Tiling => {
                    let tmp = layout::TilingLayout::new(layout::Direction::Horizontal);
                    self.layout = Box::new(tmp);
                }
            }
        }
    }

    pub fn config(&self, display: *mut xlib::Display, screen_num: libc::c_int) {
        debug!("size {}", self.windows.len());
        self.layout.configure(&self.windows, display, screen_num);
    }
}

pub struct Workspaces {
    current: char,
    pub spaces: HashMap<char, Workspace>,
}

impl Workspaces {
    pub fn new() -> Workspaces {
        Workspaces {
            current: '1',
            spaces: HashMap::new()
        }
    }

    pub fn create_workspace(&mut self, key: char) {
        if !self.spaces.contains_key(&key) {
            let space = Workspace::new();
            self.spaces.insert(key, space);
        }
    }

    pub fn delete_workspace(&mut self, key: char) {
        self.spaces.remove(&key);
    }

    pub fn get_workspace(&mut self, key: char) -> Option<&mut Workspace>{
        self.spaces.get_mut(&key)
    }

    pub fn current_workspace(&mut self) -> &mut Workspace{
        self.spaces.get_mut(&self.current).unwrap()
    }

    pub fn current_workspace_key(&self) -> char {
        self.current
    }

    pub fn switch_current(&mut self, new: char, display: *mut Display){
        if new != self.current {
            if !self.contain(new) {
                self.create(new);
            }
            let old = self.current;
            self.current = new;

            match self.get(old) {
                Some(v) => {
                    v.hide(display);
                }
                None => {}
            }

            match self.get(new) {
                Some(v) => {
                    v.show(display);
                }
                None => {}
            }
        }
    }
}
