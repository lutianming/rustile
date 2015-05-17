#![allow(unstable)]

extern crate x11;
extern crate libc;

use std::ptr;
use std::ffi;
use std::mem;
use std::str;
use std::boxed::Box;

use x11::xlib;
use x11::xlib::{ Display, Window };
use libc::{ c_int, c_ulong, c_void };


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
            let s = String::from_raw_parts(prop.value, prop.nitems as usize, prop.nitems as usize).clone();
            let text = Some(s);
            xlib::XFree(prop.value as *mut libc::c_void);
            text
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
            println!("protocols {}", count);
            for i in 0..count {
                vec = Vec::from_raw_parts(atoms, count as usize, count as usize).clone();
            }
            // xlib::XFree(atoms as *mut c_void);
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


#[cfg(test)]
mod test{
use super::*;

#[test]
fn test_open(){
    let display = open_display(None);
    assert!(display.is_some());
}

}
