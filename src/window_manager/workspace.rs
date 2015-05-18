extern crate libc;

use x11::xlib::{ Window, Display };

use std::boxed::Box;
use super::layout;
use super::super::libx;
use super::super::libx::Context;

pub struct Workspace {
    pub screen_num: libc::c_int,
    focus: Option<Window>,
    pub clean: bool,                // if workspace is not clean, it needs to be reconfig before map
    pub visible: bool,
    windows: Vec<Window>,
    layout: Box<layout::Layout>
}

impl Workspace {
    pub fn new(screen_num: libc::c_int) -> Workspace {
        Workspace {
            screen_num: screen_num,
            focus: None,
            clean: true,
            visible: false,
            windows: Vec::new(),
            layout: Box::new(layout::TilingLayout::new(layout::Direction::Horizontal))
        }
    }

    pub fn p(&self) {
        for w in self.windows.iter() {
            println!("{}", w);
        }
    }

    pub fn add(&mut self, window: Window, context: Context) -> bool {
        let mut added = false;

        // should not add root window
        if window == libx::root_window(context, self.screen_num) {
            return added;
        }

        let index = self.contain(window);
        match index {
            Some(i) => {
                // already there, do nothing
            }
            None => {
                self.windows.push(window);
                self.focus = Some(window);
                if self.visible {
                    libx::map_window(context, window);
                    self.config(context);
                    libx::set_input_focus(context, window);
                }
                added = true;
                // self.clean = false;
            }
        }
        added
    }

    pub fn remove(&mut self, window: Window, context: Context) -> bool {
        let mut removed = false;
        let index = self.contain(window);
        match index {
            Some(i) => {
                // if the window focused, change to next
                if self.focus.unwrap() == window {
                    let next = self.next_window(window);
                    if next == window {
                        // removing the last one
                        self.set_focus(None, context);
                    }
                    else{
                        self.set_focus(Some(next), context);
                    }
                }

                self.windows.remove(i);
                if self.visible {
                    libx::unmap_window(context, window);
                    self.config(context);
                }
                removed = true;
            }
            None => {}
        };
        removed
    }

    pub fn get(&self, index: usize) -> Window{
        self.windows[index]
    }

    pub fn size(&self) -> usize {
        self.windows.len()
    }

    pub fn hide(&mut self, context: Context) {
        for w in self.windows.iter() {
            libx::unmap_window(context, *w);
        }
        self.visible = false;
    }

    pub fn show(&mut self, context: Context) {
        for w in self.windows.iter() {
            libx::map_window(context, *w);
        }
        self.visible = true;

        match self.focus {
            Some(w) => {
                libx::set_input_focus(context, w);
            }
            None => {
                let root = libx::root_window(context, self.screen_num);
                libx::set_input_focus(context, root);
            }
        }
    }

    pub fn set_focus(&mut self, window: Option<Window>, context: Context) {
        match window {
            Some(w) => {
                match self.contain(w) {
                    Some(i) => {
                        self.focus = window;
                        if self.visible {
                            libx::set_input_focus(context, w);
                        }
                    }
                    None => {}
                }
            }
            None => {
                // set to root
                self.focus = window;
                if self.visible {
                    let root = libx::root_window(context, self.screen_num);
                    libx::set_input_focus(context, root);
                }
            }
        }
    }

    pub fn get_focus(&self) -> Option<Window>{
        self.focus
    }

    pub fn last_window(&mut self, window: Window) -> Window{
        match self.contain(window) {
            Some(i) => {
                let mut last = i-1;
                if last < 0 {
                    last = self.size() - 1
                }
                self.get(last)
            }
            None => {
                window
            }
        }
    }

    pub fn next_window(&self, window: Window) -> Window{
        match self.contain(window) {
            Some(i) => {
                let mut next = i+1;
                if next == self.size() {
                    next = 0;
                }
                self.get(next)
            }
            None => {
                window
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

    pub fn config(&mut self, context: Context) {
        println!("windows {}", self.windows.len());
        self.layout.configure(&self.windows, context, self.screen_num);
        self.clean = true;
    }
}
