extern crate libc;
extern crate x11;

use x11::xlib;
use std::ffi;
use std::ptr;
use std::mem;
use std::rc::{ Rc, Weak };
use std::cell::RefCell;
use std::boxed::Box;
use super::layout;
use super::super::libx;

const CWX: libc::c_uint = 1<<0;
const CWY: libc::c_uint = 1<<1;
const CWWidth: libc::c_uint = 1<<2;
const CWHeight: libc::c_uint = 1<<3;
const CWBorderWidth: libc::c_uint = 1<<4;
const CWSibling: libc::c_uint =	1<<5;
const CWStackMode: libc::c_uint = 1<<6;

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
        let mask = CWX | CWY | CWHeight | CWWidth;

        let mut change = xlib::XWindowChanges {
            x: x,
            y: y,
            width: width as i32,
            height: height as i32,
            border_width: 0,
            sibling: 0,
            stack_mode: 0
        };
        libx::configure_window(self.context, self.id, mask, change);

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

    // pub fn split(&self, index: usize) -> bool {
    //     let res = self.remove(index);
    //     match res {
    //         Some(client) => {
    //             if client.is_empty() {
    //                 let container = Container::new(self.context, self);
    //                 container.add(client);
    //                 self.clients[index] = container;
    //                 true
    //             }
    //             else{
    //                 // client is not a leaf node, can't split
    //                 false
    //             }
    //         }
    //         None => { false }
    //     }
    // }

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

    pub fn decorate(&self) {
        let attrs = libx::get_window_attributes(self.context, self.id);
        let screen = self.context.screen_num;
        let root = self.context.root;
        let display = self.context.display;
        let mut values: xlib::XGCValues = unsafe{ mem::zeroed() };
        // let gc = libx::create_gc(self.context, self.id, 0, values);
        let gc = libx::default_gc(self.context, screen);


        unsafe {
            let black = xlib::XBlackPixel(self.context.display, screen);
            let white = xlib::XWhitePixel(self.context.display, screen);

            xlib::XSetLineAttributes(self.context.display, gc, 5, 0, 0, 0);

            let cmap = xlib::XDefaultColormap(self.context.display, screen);
            let mut color: xlib::XColor = mem::zeroed();
            let name = ffi::CString::new("blue").unwrap().as_ptr();
            let r = xlib::XParseColor(self.context.display, cmap, name, &mut color);
            xlib::XAllocColor(self.context.display, cmap, &mut color);

            let (focus_id,_) = libx::get_input_focus(self.context);
            // try draw rectangle
            if focus_id == self.id || self.contain(focus_id).is_some(){
                xlib::XSetBackground(self.context.display, gc,
                                     black);
                xlib::XSetForeground(self.context.display, gc,
                                     color.pixel);
            }
            else {
                xlib::XSetBackground(self.context.display, gc,
                                     black);
                xlib::XSetForeground(self.context.display, gc,
                                     black);
            }
            let r = xlib::XFillRectangle(self.context.display,
                                         self.id, gc,
                                         1, 1,
                                         attrs.width as libc::c_uint,
                                         TITLE_HEIGHT as libc::c_uint);
        }
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
