extern crate libc;

use x11::xlib;
use std::mem;
use std::ffi;
use std::ptr;

use super::super::libx;
use super::container::{self, Container};


const CWX: libc::c_uint = 1<<0;
const CWY: libc::c_uint = 1<<1;
const CWWidth: libc::c_uint = 1<<2;
const CWHeight: libc::c_uint = 1<<3;
const CWBorderWidth: libc::c_uint = 1<<4;
const CWSibling: libc::c_uint =	1<<5;
const CWStackMode: libc::c_uint = 1<<6;


pub trait Layout {
    fn configure(&self, container: &Container);
    fn toggle(&mut self) {}
    fn get_type(&self) -> Type;
}

pub enum Direction {
    Vertical,
    Horizontal,
    Up,
    Down,
    Left,
    Right,
}

fn decorate(container: &Container, client: &Container, x: i32, y: i32, width: usize, height: usize) {
    let context = container.context;
    let attrs = libx::get_window_attributes(context, container.id);
    let screen = context.screen_num;
    let root = context.root;
    let display = context.display;
    let mut values: xlib::XGCValues = unsafe{ mem::zeroed() };
    // let gc = libx::create_gc(self.context, self.id, 0, values);
    let gc = libx::default_gc(context, screen);


    unsafe {
        let black = xlib::XBlackPixel(context.display, screen);
        let white = xlib::XWhitePixel(context.display, screen);

        xlib::XSetLineAttributes(context.display, gc, 5, 0, 0, 0);

        let cmap = xlib::XDefaultColormap(context.display, screen);
        let mut color: xlib::XColor = mem::zeroed();
        let name = ffi::CString::new("blue").unwrap().as_ptr();
        let r = xlib::XParseColor(context.display, cmap, name, &mut color);
        xlib::XAllocColor(context.display, cmap, &mut color);

        let (focus_id,_) = libx::get_input_focus(context);
        // try draw rectangle
        if focus_id == client.id {
            xlib::XSetBackground(context.display, gc,
                                 black);
            xlib::XSetForeground(context.display, gc,
                                 color.pixel);
        }
        else {
            xlib::XSetBackground(context.display, gc,
                                 black);
            xlib::XSetForeground(context.display, gc,
                                 black);
            }
        let r = xlib::XFillRectangle(context.display,
                                     container.id, gc,
                                     1, 1,
                                     attrs.width as libc::c_uint,
                                     container.titlebar_height as libc::c_uint);
        }
}

#[derive(PartialEq, Clone)]
pub enum Type {
    Tiling,
    Tab,
}

pub struct TilingLayout {
    direction: Direction,
}

pub struct TabLayout;


impl TabLayout {
    pub fn new() -> TabLayout{
        TabLayout
    }
}

impl Layout for TabLayout {
    fn get_type(&self) -> Type { Type::Tab }
    fn configure(&self, container: &Container) {
        let size = container.clients.len();
        if size == 0{
            return;
        }

        let attrs = libx::get_window_attributes(container.context, container.id);
        let width = attrs.width/size as i32;
        let (focus_id,_) = libx::get_input_focus(container.context);

        for (i, client) in container.clients.iter().enumerate() {
            decorate(container, client,
                     0, width*i as i32,
                     width as usize,
                     container.titlebar_height as usize);
            if focus_id == client.id {
                client.configure(0, 0, attrs.width as usize, attrs.height as usize);
                client.map();
            }
            else{
                client.unmap();
            }
        }
    }
}

impl TilingLayout {
    pub fn new(d: Direction) -> TilingLayout{
        TilingLayout {
            direction: d,
        }
    }
}

impl Layout for TilingLayout {
    fn get_type(&self) -> Type { Type::Tiling }
    fn toggle(&mut self) {
        match self.direction {
            Direction::Vertical => self.direction = Direction::Horizontal,
            Direction::Horizontal => self.direction = Direction::Vertical,
            _ => {}
        }
    }
    /// once we add or remove a window, we need to reconfig
    fn configure(&self, container: &Container) {
        let size = container.clients.len();
        if size == 0 {
            return;
        }

        let attrs = libx::get_window_attributes(container.context, container.id);

        let width = attrs.width  / size as libc::c_int;
        let height = attrs.height / size as libc::c_int;

        for (i, client) in container.clients.iter().enumerate() {
            let mut x = 0;
            let mut y = 0;
            let mut w = attrs.width;
            let mut h = attrs.height;

            match self.direction {
                Direction::Vertical => {
                    y = height * i as libc::c_int;
                    h = height;
                }
                Direction::Horizontal => {
                    x = width * i as libc::c_int;
                    w = width;
                }
                _ => {}
            };

            decorate(container, client,
                     x, y,
                     w as usize,
                     container.titlebar_height as usize);
            client.configure(x, y+container.titlebar_height as i32, w as usize, h as usize);
        }
    }
}
