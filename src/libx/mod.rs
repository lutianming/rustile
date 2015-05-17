#![allow(unstable)]

extern crate x11;
extern crate libc;

use std::ptr;
use std::ffi;
use std::mem;
use std::str;
use std::slice;
use std::boxed::Box;

use x11::xlib;
use x11::xlib::{ Display, Window };
use libc::{ c_int, c_long, c_uint, c_ulong, c_void };


pub fn open_display(name: Option<&str>) -> Option<*mut Display> {
    unsafe{
        let display = match name {
            Some(n) => {
                let s = ffi::CString::new(n).unwrap();
                xlib::XOpenDisplay(s.as_ptr())
            }
            None => { xlib::XOpenDisplay(ptr::null()) }
        };

        if display == ptr::null_mut() {
            None
        }
        else{
            // let b = Some(Box::<Display>::new(*display));
            // xlib::XFree(display as *mut libc::c_void);
            // b
            Some(display)
        }
    }
}

pub fn close_display(display: *mut Display) {
    unsafe{
        xlib::XCloseDisplay(display);
    }
}

pub fn default_screen(display: *mut Display) -> c_int {
    unsafe{
        xlib::XDefaultScreen(display)
    }
}

pub fn root_window(display: *mut Display, screen_num: c_int) -> c_ulong {
    unsafe {
        xlib::XRootWindow(display, screen_num)
    }
}

pub fn next_event(display: *mut Display) -> xlib::XEvent {
    unsafe{
        let mut e: xlib::XEvent = mem::zeroed();
        xlib::XNextEvent(display, &mut e);
        e
    }
}

pub fn get_atom_name(display: *mut Display, atom: xlib::Atom) -> Option<String> {
    unsafe{
        let name = xlib::XGetAtomName(display, atom);
        if name == ptr::null_mut() {
            return None
        }
        let s = ffi::CStr::from_ptr(name);
        let s = s.to_bytes();
        match str::from_utf8(s) {
            Ok(v) => {
                let tmp = Some(v.to_string());
                xlib::XFree(name as *mut libc::c_void);
                tmp
            }
            _ => None
        }
    }
}

pub fn get_atom(display: *mut Display, name: &str) -> xlib::Atom{
    unsafe{
        let cstr = ffi::CString::new(name).unwrap();
        let atom = xlib::XInternAtom(display, cstr.as_ptr(), xlib::False);
        atom
    }
}

pub fn get_text_property(display: *mut Display, window: xlib::Window, atom: xlib::Atom) -> Option<String>{
    unsafe{
        let mut prop: xlib::XTextProperty = mem::zeroed();
        let r = xlib::XGetTextProperty(display, window, &mut prop, atom);
        if r == 0 || prop.nitems == 0{
            None
        }else{

            // let s = String::from_raw_parts(prop.value, prop.nitems as usize, prop.nitems as usize).clone()
            let s = slice::from_raw_parts(prop.value, prop.nitems as usize).iter().map(|&c| c as u8).collect();
            match String::from_utf8(s) {
                Ok(v) => {
                    xlib::XFree(prop.value as *mut libc::c_void);
                    Some(v)
                }
                _ => None
            }
        }
    }
}

pub fn get_wm_protocols(display: *mut Display, window: Window) -> Vec<xlib::Atom>{
    unsafe{
        let mut atoms: *mut xlib::Atom = ptr::null_mut();
        let mut count = 0;
        let s = xlib::XGetWMProtocols(display, window, &mut atoms, &mut count);
        let mut vec: Vec<xlib::Atom> = Vec::new();
        if s > 0 {
            vec = slice::from_raw_parts(atoms, count as usize).iter()
                .map(|&a| a as xlib::Atom).collect();
            xlib::XFree(atoms as *mut c_void);
        }
        vec
    }
}

pub fn get_window_attributes(display: *mut Display, window: Window) -> xlib::XWindowAttributes {
    unsafe{
        let mut attrs: xlib::XWindowAttributes = mem::zeroed();
        xlib::XGetWindowAttributes(display, window, &mut attrs);
        attrs
    }
}

pub fn get_transient_for_hint(display: *mut Display, window: Window) -> i32 {
    unsafe{
        let mut window_return: xlib::Window = 0;
        xlib::XGetTransientForHint(display, window, &mut window_return)
    }
}

pub fn change_window_attributes(display: *mut Display, window: Window, valuemask: c_ulong, attrs: *mut xlib::XSetWindowAttributes) {
    unsafe{
        xlib::XChangeWindowAttributes(display, window, valuemask, attrs);
    }

}

fn get_window_property(display: *mut xlib::Display, window: Window, atom: xlib::Atom) {
    unsafe {
        let mut actual_type_return: u64 = 0;
        let mut actual_format_return: i32 = 0;
        let mut nitem_return: libc::c_ulong = 0;
        let mut bytes_after_return: libc::c_ulong = 0;
        let mut prop_return: *mut libc::c_uchar = mem::zeroed();

        let r = xlib::XGetWindowProperty(display, window, xlib::XA_WM_NAME,
                                         0, 0xFFFFFFFF, 0, 0,
                                         &mut actual_type_return,
                                         &mut actual_format_return,
                                         &mut nitem_return,
                                         &mut bytes_after_return,
                                         &mut prop_return);

        debug!("result get wp {}", r);
        if r == 0 {

        }else{
            debug!("actual format return {}", actual_format_return);
            if actual_format_return == 0 {

            }
            else {
                // let s = from_raw_parts(prop_return as *const libc::c_ulong, nitems_return as usize).iter()
                //     .map(|&c| c as u64)
                //     .collect();
                let s = String::from_raw_parts(prop_return, nitem_return as usize, nitem_return as usize);
                println!("{}", s);

            }
        }
    }
}

pub fn set_input_focus(display: *mut Display, window: Window) {
    let none = 0;
    let pointer_root = 1;
    let parent = 2;
    unsafe{
        xlib::XSetInputFocus(display, window, pointer_root, 0);
    }
}

pub fn get_input_focus(display: *mut Display) -> (Window, c_int){
    let mut window: xlib::Window = 0;
    let mut revert_to: libc::c_int = 0;
    unsafe{
        let s = xlib::XGetInputFocus(display, &mut window, &mut revert_to);
    }
    (window, revert_to)
}

pub fn unmap_window(display: *mut Display, window: Window) -> c_int{
    unsafe{
        xlib::XUnmapWindow(display, window)
    }

}

pub fn configure_window(display: *mut Display, window: Window, mask:c_uint, mut change: xlib::XWindowChanges) {
    unsafe{
        xlib::XConfigureWindow(display, window, mask, &mut change);
    }
}
pub fn map_window(display: *mut Display, window: Window) -> c_int{
    unsafe{
        xlib::XMapWindow(display, window)
    }
}

pub fn kill_window(display: *mut Display, window: Window) {
    let mut event: xlib::XClientMessageEvent = unsafe {
        mem::zeroed()
    };

    let wm_delete_window = get_atom(display, "WM_DELETE_WINDOW");
    let wm_protocols = get_atom(display, "WM_PROTOCOLS");
    let protocols = get_wm_protocols(display, window);

    if protocols.iter().any(|&p| p == wm_delete_window){
        event.type_ = xlib::ClientMessage;
        event.message_type = wm_protocols;
        event.format = 32;
        event.window = window;
        event.send_event = xlib::True;
        event.display = display;
        event.data.set_long(0, wm_delete_window as libc::c_long);

        let mut e: xlib::XEvent = From::from(event);
        send_event(display, window, xlib::True, xlib::NoEventMask, e);
    }
    else{
        unsafe{
            xlib::XKillClient(display, window);
        }
    }

}

pub fn send_event(display: *mut Display, window: Window,
                  propagate: c_int, event_mask: c_long,
                  mut event: xlib::XEvent) {
    unsafe{
        xlib::XSendEvent(display, window, propagate, event_mask, &mut event);
    }
}

pub fn string_to_keysym(s: &str) -> xlib::KeySym {
    let tmp = ffi::CString::new(s).unwrap();
    unsafe{
        xlib::XStringToKeysym(tmp.as_ptr())
    }

}

#[cfg(test)]
mod test{
use super::*;

#[test]
fn test_open(){
    let display = open_display(None);
    assert!(display.is_some());
}

}
