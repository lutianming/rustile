extern crate libc;
extern crate x11;

use x11::xlib;
use super::super::libx;

const CWX: libc::c_uint = 1<<0;
const CWY: libc::c_uint = 1<<1;
const CWWidth: libc::c_uint = 1<<2;
const CWHeight: libc::c_uint = 1<<3;
const CWBorderWidth: libc::c_uint = 1<<4;
const CWSibling: libc::c_uint =	1<<5;
const CWStackMode: libc::c_uint = 1<<6;

const TITLE_HEIGHT: libc::c_int = 20;

#[derive(Debug, Copy, Clone)]
pub struct Window {
    pub id: xlib::Window,
    pub client: Option<xlib::Window>,
    context: libx::Context,
}

impl PartialEq for Window {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Window {
    pub fn new(context: libx::Context, id: xlib::Window) -> Window{
        Window {
            context: context,
            client: None,
            id: id
        }
    }

    pub fn root(context: libx::Context, screen_num: libc::c_int) -> Window {
        let root = libx::root_window(context, screen_num);
        Window {
            context: context,
            client: None,
            id: root,
        }
    }

    pub fn decorate(context: libx::Context, id: xlib::Window) -> Window {
        let res = libx::query_tree(context, id);

        match res {
            Some((root, _, _)) => {
                let attrs = libx::get_window_attributes(context, id);
                let parent = libx::create_window(context, root, attrs.x, attrs.y, attrs.width as libc::c_uint, attrs.height as libc::c_uint);
                libx::select_input(context, parent, attrs.all_event_masks| xlib::SubstructureNotifyMask | xlib::SubstructureRedirectMask);
                libx::reparent(context, id, parent, 0, TITLE_HEIGHT);

                Window {
                    context: context,
                    client: Some(id),
                    id: parent
                }
            }
            None => {
                Window {
                    context: context,
                    client: None,
                    id: id
                }
            }
        }
    }

    pub fn get_top(&self) -> Window{
        let top_id = libx::get_top_window(self.context, self.id);
        Window {
            context: self.context,
            client: Some(self.id),
            id: top_id.unwrap()
        }
    }

    pub fn is_top(&self) -> bool {
        self.get_top().id == self.id
    }

    pub fn can_manage(&self) -> bool {
        let attrs = libx::get_window_attributes(self.context, self.id);
        let transientfor_hint = libx::get_transient_for_hint(self.context, self.id);
        attrs.override_redirect == 0 && transientfor_hint == 0
    }

    pub fn configure(&self, x: i32, y: i32, width: i32, height: i32) {
        let mask = CWX | CWY | CWHeight | CWWidth;

        let mut change = xlib::XWindowChanges {
            x: x,
            y: y,
            width: width,
            height: height,
            border_width: 0,
            sibling: 0,
            stack_mode: 0
        };
        libx::configure_window(self.context, self.id, mask, change);

        match self.client {
            Some(c) => {
                let w = Window::new(self.context, c);
                println!("{} {} {}", c, width, height-TITLE_HEIGHT);
                w.configure(0, TITLE_HEIGHT, width, height-TITLE_HEIGHT);

            }
            None => {}
        }
    }

    pub fn map(&self) {
        libx::map_window(self.context, self.id);
        unsafe{
            if self.client.is_some(){
                xlib::XMapSubwindows(self.context.display, self.id);
            }

        }
    }

    pub fn unmap(&self) {
        unsafe {
            if self.client.is_some(){
                xlib::XUnmapSubwindows(self.context.display, self.id);                 }
        }
        libx::unmap_window(self.context, self.id);
    }

    pub fn focus(&self) {
        libx::set_input_focus(self.context, self.id);
    }

    fn draw_titlebar(&self) {
        let attrs = libx::get_window_attributes(self.context, self.id);
        unsafe {

        }
    }
}

#[test]
fn window_eq() {
    use std::ptr;
    let c1 = libx::Context {
        display: ptr::null_mut()
    };
    let w1 = Window::new(c1, 1);
    let c2 = libx::Context {
        display: ptr::null_mut()
    };
    let w2 = Window::new(c2, 1);
    assert_eq!(w1, w2);
}
