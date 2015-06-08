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

const border: u32 = 1;

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
pub enum LayoutDirection {
    Vertical,
    Horizontal,
}

#[derive(PartialEq, Clone)]
pub enum MoveDirection {
    Up,
    Down,
    Left,
    Right,
}

pub fn decorate(client: &Container, focused: bool) {
    match client.titlebar {
        Some(rec) => {
            set_titlebar(client, rec, focused);
            set_title(client, rec, focused);
        }
        None => {}
    }
    set_border(client, focused);
}

fn set_titlebar(client: &Container, rec: Rectangle, focused: bool) {
    let mut context = client.context;

    context.gc = if focused {
        context.focus_gc
    }
    else {
        context.unfocus_gc
    };

    let pid = client.pid();
    libx::fill_rectangle(context, pid,
                         rec.x, rec.y,
                         rec.width, rec.height);
}

fn set_title(client: &Container, rec: Rectangle, focused: bool) {
    let mut context = client.context;

    context.gc = if focused {
        context.focus_font_gc
    }
    else {
        context.unfocus_font_gc
    };

    let res = libx::get_text_property(context, client.raw_id(), xlib::XA_WM_NAME);

    match res {
        Some(s) => {

            let (boundingbox, dummy) = libx::text_extents(context, s.clone());

            let offset_x = -dummy.x as i32;
            let offset_y = -dummy.y as i32;

            let x = rec.x+offset_x;
            let y = rec.y+offset_y;
            let pid = client.pid();

            libx::draw_string(context, s, pid, x, y);
        }
        None =>{}
    }
}

fn set_border(client: &Container, focused: bool) {
    let mut context = client.context;

    context.gc = if focused {
        context.focus_gc
    }
    else{
        context.unfocus_gc
    };
    unsafe {
        xlib::XSetLineAttributes(context.display, context.gc, border, 0, 0, 0);
    }
    let pid = client.pid();
    let rec = client.rec();

    libx::draw_rectangle(context, pid,
                         rec.x-border as i32,
                         rec.y-border as i32,
                         rec.width+border*2-1 as u32,
                         rec.height+border*2-1 as u32);
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
    let size = container.size() as u32;
    if size == 0 {
        return;
    }

    let attrs = container.rec();

    let x_offset = 0; //attrs.x;
    let y_offset = 0; //attrs.y;

    let (focus_id,_) = libx::get_input_focus(container.context);

    let mut x = x_offset;
    let mut y = y_offset;
    let mut w: u32 = 0;
    let mut h: u32 = 0;
    for (i, client) in container.clients.iter_mut().enumerate() {
        let id = client.raw_id();

        match container.direction {
            LayoutDirection::Vertical => {
                y = y + h as i32;
                h = (attrs.height as f32 * client.portion) as u32;
                w = attrs.width;
            }
            LayoutDirection::Horizontal => {
                x = x + w as i32;
                w = (attrs.width as f32 * client.portion) as u32;
                h = attrs.height;
            }
        };

        client.titlebar = Some(Rectangle {
            x: x,
            y: y,
            width: w,
            height: client.titlebar_height,
        });

        let titlebar_height = client.titlebar.unwrap().height;
        decorate(client, id==focus_id);;

        client.configure(x+border as i32,
                         y+titlebar_height as i32 + border as i32,
                         w-border*2, h-titlebar_height-border*2);
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

        client.configure(x_offset + border as i32,
                         y_offset+titlebar_height as i32 + border as i32,
                         attrs.width - border*2 ,
                         attrs.height -titlebar_height - border*2);
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
