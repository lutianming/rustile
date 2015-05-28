extern crate libc;
extern crate x11;

use x11::xlib;

use std::rc::{ Rc, Weak };
use std::cell::RefCell;
use std::boxed::Box;
use super::layout;
use super::super::libx;

pub const TITLE_HEIGHT: libc::c_int = 20;

pub enum Mode {
    Normal,
    Fullscreen,
}

pub struct Container {
    pub id: xlib::Window,
    pub visible: bool,
    pub titlebar_height: usize,
    pub pid: Option<xlib::Window>,
    pub clients: Vec<Container>,
    pub mode: Mode,
    pub context: libx::Context,
    layout: Box<layout::Layout>,
}

impl PartialEq for Container {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}


impl Container {
    pub fn new(context: libx::Context, parent: Option<xlib::Window>) -> Container{
        let pid = match parent {
            Some(c) => {
                c
            }
            None => {
                context.root
            }
        };
        let attrs = libx::get_window_attributes(context, pid);

        let id = libx::create_window(context, pid,
                                     attrs.x, attrs.y,
                                     attrs.width as libc::c_uint,
                                     attrs.height as libc::c_uint);
        libx::select_input(context, id, xlib::SubstructureNotifyMask | xlib::SubstructureRedirectMask);
        let mut c = Container::from_id(context, id);
        c.pid = Some(pid);
        c
    }

    pub fn from_id(context: libx::Context, id: xlib::Window) -> Container {
        Container {
            context: context,
            clients: Vec::new(),
            pid: None,
            visible: false,
            id: id,
            mode: Mode::Normal,
            titlebar_height: 0,
            layout: Box::new(layout::TilingLayout::new(layout::Direction::Horizontal))
        }
    }

    pub fn print_tree(&self, indent: i32) {
        for i in 0..indent {
            print!(" ");
        }
        let attrs = libx::get_window_attributes(self.context, self.id);
        println!("C {} pid:{} x:{} y:{} w:{} h:{}", self.id, self.pid.unwrap_or(0), attrs.x, attrs.y, attrs.width, attrs.height);
        for client in self.clients.iter() {
            client.print_tree(indent+4);
        }
    }

    // pub fn is_top(&self) -> bool {
    //     let top = self.get_top();
    //     match top {
    //         Some(w) => {
    //             w.id == self.id
    //         }
    //         Noen => false
    //     }
    // }

    pub fn is_empty(&self) -> bool{
        self.clients.is_empty()
    }

    pub fn size(&self) -> usize {
        self.clients.len()
    }
    pub fn add(&mut self, mut client: Container) {
        if client.pid.is_none() || client.pid.unwrap() != self.id {
            libx::reparent(self.context, client.id, self.id, 0, 0);
            client.pid = Some(self.id);
        }
        self.clients.push(client);
    }

    pub fn remove(&mut self, id: xlib::Window) -> Option<Container>{
        let res = self.contain(id);
        match res {
            Some(index) => {
                let r = self.clients.remove(index);
                Some(r)
            }
            None => {
                for c in self.clients.iter_mut() {
                    let r = c.remove(id);
                    if r.is_some(){
                        return r;
                    }
                }
                None
            }
        }
    }

    pub fn get(&self, index: usize) -> Option<&Container> {
        self.clients.get(index)
    }
    pub fn contain(&self, id: xlib::Window) -> Option<usize>{
        self.clients.iter().position(|x| (*x).id == id)
    }

    pub fn configure(&self, x: i32, y: i32, width: usize, height: usize) {
        libx::resize_window(self.context, self.id, x, y, width, height);
        // layout for children clients
        self.update();
    }

    pub fn update(&self) {
        self.layout.configure(self);
    }

    pub fn map(&self) {
        // self.visible = true;
        libx::map_window(self.context, self.id);
        for client in self.clients.iter() {
            client.map();
        }
    }

    pub fn unmap(&self) {
        // self.visible = false;
        for client in self.clients.iter() {
            client.unmap();
        }
        libx::unmap_window(self.context, self.id);
    }

    pub fn destroy(&self) -> bool {
        // can distroy only if it has no clients
        if self.clients.is_empty() {
            unsafe{
                xlib::XDestroyWindow(self.context.display, self.id);
                true
            }
        }
        else {
            false
        }
    }

    pub fn next_client(&self, id: xlib::Window) -> Option<&Container>{
        match self.contain(id) {
            Some(i) => {
                let mut next = i+1;
                if next == self.size() {
                    next = 0;
                }
                self.get(next)
            }
            None => {
                None
            }
        }
    }

    pub fn last_client(&self, id: xlib::Window) -> Option<&Container>{
        match self.contain(id) {
            Some(i) => {
                let mut last = i-1;
                if last < 0 {
                    last = self.size() - 1
                }
                self.get(last)
            }
            None => {
                None
            }
        }
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
                layout::Type::Tab => {
                    let tmp = layout::TabLayout::new();
                    self.layout = Box::new(tmp);
                }
            }
        }
    }

    pub fn split(&mut self) -> bool {
        if !self.is_empty() {
            return false;
        }

        let mut container = Container::new(self.context, None);
        let id = self.id;
        self.id = container.id;
        container.id = id;
        self.add(container);
        true
    }

    pub fn focus(&self) {
        libx::set_input_focus(self.context, self.id);
        // match self.client {
        //     Some(id) => {
        //         libx::set_input_focus(self.context, id);
        //         self.draw_titlebar(true);
        //     }
        //     None => {
        //         libx::set_input_focus(self.context, self.id);
        //     }
        // }
    }

    pub fn unfocus(&self) {

    }

    pub fn switch_client(&self) {
        let (x, _) = libx::get_input_focus(self.context);

    }
}

// #[test]
// fn window_eq() {
//     use std::ptr;
//     let c1 = libx::Context {
//         display: ptr::null_mut()
//     };
//     let w1 = Window::new(c1, 1);
//     let c2 = libx::Context {
//         display: ptr::null_mut()
//     };
//     let w2 = Window::new(c2, 1);
//     assert_eq!(w1, w2);
// }
