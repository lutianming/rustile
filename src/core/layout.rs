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

#[derive(Copy, Clone)]
pub struct Rectangle {
    pub x: i32,
    pub y: i32,
    pub width: usize,
    pub height: usize,
}

impl Rectangle {
    pub fn contain(&self, x: i32, y: i32) -> bool{
        if x >= self.x && x <= (self.x + self.width as i32) && y >= self.y && y <= (self.y + self.height as i32) {
            true
        }
        else{
            false
        }
    }
}
#[derive(PartialEq, Clone)]
pub enum Direction {
    Vertical,
    Horizontal,
    Up,
    Down,
    Left,
    Right,
}

pub fn decorate_focus(client: &Container) {
    let context = client.context;
    let gc = context.gc;

    let pid = match client.get_parent() {
        Some(p) => { p.id }
        None => { context.root }
    };

    match client.titlebar {
        Some(rec) => {
            unsafe{
                xlib::XSetBackground(context.display, gc,
                                     context.focus_bg);
                xlib::XSetForeground(context.display, gc,
                                     context.focus_fg);

                let r = xlib::XFillRectangle(context.display,
                                             pid, gc,
                                             rec.x, rec.y,
                                             rec.width as u32,
                                             rec.height as u32);
            }

        }
        None => {}
    }
}

pub fn decorate_unfocus(client: &Container) {
    let context = client.context;
    let gc = context.gc;

    let pid = match client.get_parent() {
        Some(p) => { p.id }
        None => { context.root }
    };

    match client.titlebar {
        Some(rec) => {
            unsafe {
                xlib::XSetBackground(context.display, gc,
                                     context.unfocus_bg);
                xlib::XSetForeground(context.display, gc,
                                     context.unfocus_fg);

                let r = xlib::XFillRectangle(context.display,
                                             pid, gc,
                                             rec.x, rec.y,
                                             rec.width as u32,
                                             rec.height as u32);
            }
        }
        None => {}
    }
}

pub fn decorate(client: &Container) {
    let (focus_id,_) = libx::get_input_focus(client.context);
    println!("focus id {}, client id {}", focus_id, client.id);

    // try draw rectangle
    if focus_id == client.id {
        decorate_focus(client);
    }
    else {
        decorate_unfocus(client);
    }
}

#[derive(PartialEq, Clone)]
pub enum Type {
    Tiling,
    Tab,
}

pub fn update_layout(container: &mut Container) {
    match container.layout {
        Type::Tiling => {
            layout_tiling(container);
        }
        Type::Tab => {
            layout_tab(container);
        }
    }
}

fn layout_tiling(container: &mut Container) {
    let size = container.clients.len();
    if size == 0 {
        return;
    }

    let attrs = libx::get_window_attributes(container.context, container.id);

    let width = attrs.width  / size as libc::c_int;
    let height = attrs.height / size as libc::c_int;

    for (i, client) in container.clients.iter_mut().enumerate() {
        let mut x = 0;
        let mut y = 0;
        let mut w = attrs.width;
        let mut h = attrs.height;

        match container.direction {
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

        client.titlebar = Some(Rectangle {
            x: x,
            y: y,
            width: w as usize,
            height: client.titlebar_height,
        });

        let titlebar_height = client.titlebar.unwrap().height;
        decorate(client);;

        h = h - titlebar_height as i32;
        client.configure(x, y+titlebar_height as i32,
                         w as usize, h as usize);
        client.map();
    }
}

fn layout_tab(container: &mut Container) {
    let size = container.clients.len();
    if size == 0{
        return;
    }

    let attrs = libx::get_window_attributes(container.context, container.id);
    let width = attrs.width/size as i32;
    let (focus_id,_) = libx::get_input_focus(container.context);

    for (i, client) in container.clients.iter_mut().enumerate() {
        client.titlebar = Some(Rectangle {
            x: 0,
            y: width*i as i32,
            width: width as usize,
            height: client.titlebar_height,
        });

        let titlebar_height = client.titlebar.unwrap().height;
        decorate(client);

        if focus_id == client.id {
            client.configure(0, titlebar_height as i32,
                             attrs.width as usize,
                             (attrs.height-titlebar_height as i32) as usize);
            client.map();
        }
        else{
            client.unmap();
        }
    }
}
