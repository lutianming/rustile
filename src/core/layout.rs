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

fn decorate_focus(pid: xlib::Window, client: &Container, x: i32, y: i32, width: usize, height: usize) {
    let context = client.context;
    let gc = context.gc;
    unsafe{
        xlib::XSetBackground(context.display, gc,
                             context.focus_bg);
        xlib::XSetForeground(context.display, gc,
                             context.focus_fg);

        let r = xlib::XFillRectangle(context.display,
                                     pid, gc,
                                     x, y,
                                     width as u32, height as u32);
    }

}

fn decorate_unfocus(pid: xlib::Window, client: &Container, x: i32, y: i32, width: usize, height: usize) {
    let context = client.context;
    let gc = context.gc;
    unsafe {
        xlib::XSetBackground(context.display, gc,
                             context.unfocus_bg);
        xlib::XSetForeground(context.display, gc,
                             context.unfocus_fg);

        let r = xlib::XFillRectangle(context.display,
                                     pid, gc,
                                     x, y,
                                     width as u32, height as u32);
    }
}

fn decorate(pid: xlib::Window, client: &Container, x: i32, y: i32, width: usize, height: usize) {
    let (focus_id,_) = libx::get_input_focus(client.context);
    println!("focus id {}, client id {}", focus_id, client.id);

    // try draw rectangle
    if focus_id == client.id {
        decorate_focus(pid, client, x, y, width, height);
    }
    else {
        decorate_unfocus(pid, client, x, y, width, height);
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
            decorate(container.id, client,
                     0, width*i as i32,
                     width as usize,
                     client.titlebar_height as usize);
            if focus_id == client.id {
                client.configure(0, client.titlebar_height as i32, attrs.width as usize, (attrs.height-client.titlebar_height as i32) as usize);
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

            decorate(container.id, client,
                     x, y,
                     w as usize,
                     client.titlebar_height as usize);
            h = h - client.titlebar_height as i32;
            client.configure(x, y+client.titlebar_height as i32, w as usize, h as usize);
            client.map();
        }
    }
}
