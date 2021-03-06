extern crate x11;

use x11::xlib;
use std::mem;
use super::super::libx;
pub struct TaskBar {
    context: libx::Context,
    id: xlib::Window,
    height: u32,
    current: Option<char>,
    workspaces: Vec<char>,
}

impl TaskBar {
    pub fn new(context: libx::Context, height: u32, position: i32) -> TaskBar {
        let pid = context.root;
        let attrs = libx::get_window_attributes(context, pid);
        let y: i32 = if position > 0 {
            attrs.y
        }else{
            attrs.y + attrs.height - height as i32
        };
        let id = libx::create_window(context, pid, attrs.x, y,
                                     attrs.width as u32,
                                     height);

        // attributes
        let mut attrs: xlib::XSetWindowAttributes = unsafe { mem::zeroed() };
        attrs.override_redirect = 1;
        libx::set_window_attributes(context, id, xlib::CWOverrideRedirect, attrs);

        // inputs
        let mask = xlib::ButtonPressMask | xlib::ExposureMask;
        libx::select_input(context, id, mask);
        libx::map_window(context, id);
        TaskBar {
            context: context,
            id: id,
            height: height,
            current: None,
            workspaces: Vec::new()
        }
    }

    pub fn load(&mut self, keys: Vec<char>) {
        self.workspaces = keys;
    }

    pub fn set_current(&mut self, current: char) {
        self.current = Some(current);
    }

    pub fn update(&mut self) {
        let mut context = self.context;
        let gc = context.gc;
        let display = context.display;
        for (i, v) in self.workspaces.iter().enumerate() {
            let x = (i as u32 * (self.height + 1)) as i32 + 1;
            let y = 1;
            let width = self.height - 2;
            let height = self.height - 2;
            unsafe{
                let is_current = self.current.is_some() && v.clone() == self.current.unwrap();
                context.gc = if is_current {
                    context.focus_gc
                }
                else {
                    context.unfocus_gc
                };

                libx::fill_rectangle(context, self.id,
                                     x, y,
                                     width, height);

                context.gc = if is_current {
                    context.focus_font_gc
                }
                else{
                    context.unfocus_font_gc
                };

                let s = v.to_string();

                let (boundingbox, dummy) = libx::text_extents(context, s.clone());
                let offset_x = (width as i32- dummy.width as i32)/2 - dummy.x as i32;
                let offset_y = (height as i32 - dummy.height as i32)/2 - dummy.y as i32;
                libx::draw_string(context, s, self.id,
                                  x+offset_x, y+offset_y);
            }
        }
    }

    pub fn handle(&mut self, e: &xlib::XEvent) {
        let t = e.get_type();
        match t {
            xlib::Expose => {
                let event: xlib::XExposeEvent = From::from(*e);
                if event.window == self.id {
                    self.update();
                }
            }
            _ => {}
        }
    }
}
