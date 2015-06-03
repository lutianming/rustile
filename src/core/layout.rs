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

#[derive(Debug, Copy, Clone)]
pub struct Rectangle {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
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

pub fn decorate(client: &Container, focused: bool) {
    let context = client.context;
    let gc = context.gc;

    let pid = match client.get_parent() {
        Some(p) => { p.raw_id() }
        None => { context.root }
    };

    match client.titlebar {
        Some(rec) => {
            unsafe{
                if focused {
                    xlib::XSetBackground(context.display, gc,
                                         context.focus_bg);
                    xlib::XSetForeground(context.display, gc,
                                         context.focus_fg);
                }
                else{
                    xlib::XSetBackground(context.display, gc,
                                         context.unfocus_bg);
                    xlib::XSetForeground(context.display, gc,
                                         context.unfocus_fg);                            }

                let r = xlib::XFillRectangle(context.display,
                                             pid, gc,
                                             rec.x, rec.y,
                                             rec.width as u32,
                                             rec.height as u32);

                if focused {
                    xlib::XSetBackground(context.display, gc,
                                         context.focus_fg);
                    xlib::XSetForeground(context.display, gc,
                                         context.font_color);
                }
                else {
                    xlib::XSetBackground(context.display, gc,
                                         context.unfocus_fg);
                    xlib::XSetForeground(context.display, gc,
                                         context.font_color);
                }

                let offset_x = 10;
                let offset_y = 10;

                let res = libx::get_text_property(context, client.raw_id(), xlib::XA_WM_NAME);
                match res {
                    Some(s) => {
                        println!("window {} {}", client.raw_id(), s);
                        // let s = "标题";
                        let size = s.len() as i32;
                        let title = ffi::CString::new(s).unwrap().as_ptr();

                        let r = xlib::XmbDrawString(context.display, pid,
                                                    context.fontset, gc,
                                                    rec.x+offset_x, rec.y+offset_y,
                                                    title, size);
                    }
                    None =>{}
                }

            }

        }
        None => {}
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
    let size = container.clients.len() as u32;
    if size == 0 {
        return;
    }

    let attrs = container.rec();

    let width = attrs.width  / size;
    let height = attrs.height / size;
    let x_offset = 0; //attrs.x;
    let y_offset = 0; //attrs.y;

    let (focus_id,_) = libx::get_input_focus(container.context);

    for (i, client) in container.clients.iter_mut().enumerate() {
        let id = client.raw_id();
        let mut x = x_offset;
        let mut y = y_offset;
        let mut w = attrs.width;
        let mut h = attrs.height;

        match container.direction {
            Direction::Vertical => {
                y = y + (height * i as u32) as i32;
                h = height;
            }
            Direction::Horizontal => {
                x = x + (width * i as u32) as i32;
                w = width;
            }
            _ => {}
        };

        client.titlebar = Some(Rectangle {
            x: x,
            y: y,
            width: w,
            height: client.titlebar_height,
        });

        let titlebar_height = client.titlebar.unwrap().height;
        decorate(client, id==focus_id);;

        h = h - titlebar_height;
        client.configure(x, y+titlebar_height as i32,
                         w, h);
        // client.map();
    }
}

fn layout_tab(container: &mut Container) {
    let size = container.size() as u32;
    if size == 0{
        return;
    }

    let attrs = container.rec();

    let width = attrs.width / size;
    let x_offset = 0; //attrs.x;
    let y_offset = 0; //attrs.y;
    let (focus_id,_) = libx::get_input_focus(container.context);

    for (i, client) in container.clients.iter_mut().enumerate() {
        let id = client.raw_id();
        client.titlebar = Some(Rectangle {
            x: x_offset + (width * i as u32) as i32,
            y: y_offset as i32,
            width: width,
            height: client.titlebar_height,
        });

        let titlebar_height = client.titlebar.unwrap().height;
        decorate(client, id==focus_id);

        client.configure(x_offset, y_offset+titlebar_height as i32,
                         attrs.width,
                         attrs.height -titlebar_height);
        if focus_id == id {
            // client.map();
            libx::raise_window(client.context, id);
        }
        else{
            // client.unmap();
            // libx::lower_window(client.context, client.id);
        }
    }
}
