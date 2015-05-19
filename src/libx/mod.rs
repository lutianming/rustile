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

#[derive(Debug, Copy, Clone)]
pub struct Context {
    display: *mut Display
}

pub fn open_display(name: Option<&str>) -> Option<Context> {
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
            let c = Context {
                display: display,
            };
            Some(c)
        }
    }
}

pub fn close_display(context: Context) {
    unsafe{
        xlib::XCloseDisplay(context.display);
    }
}

pub fn default_screen(context: Context) -> c_int {
    unsafe{
        xlib::XDefaultScreen(context.display)
    }
}

pub fn root_window(context: Context, screen_num: c_int) -> c_ulong {
    unsafe {
        xlib::XRootWindow(context.display, screen_num)
    }
}

pub fn next_event(context: Context) -> xlib::XEvent {
    unsafe{
        let mut e: xlib::XEvent = mem::zeroed();
        xlib::XNextEvent(context.display, &mut e);
        e
    }
}

pub fn define_cursor(context: Context, window: Window, shape: c_uint) {
    unsafe {
        let cursor = xlib::XCreateFontCursor(context.display, shape);
        xlib::XDefineCursor(context.display, window, cursor);
    }
}

pub fn grab_key(context: Context, keycode: xlib::KeyCode, modifiers: c_uint, window: Window) {
    unsafe {
        xlib::XGrabKey(context.display, keycode as c_int, modifiers, window, 1, xlib::GrabModeAsync, xlib::GrabModeAsync);
    }
}

pub fn grab_button(context: Context, button: c_uint, modifiers: c_uint, window: Window) {
    unsafe {
        xlib::XGrabButton(context.display, button, modifiers, window,
                          0,
                          xlib::ButtonPressMask as c_uint,
                          xlib::GrabModeAsync, xlib::GrabModeAsync,
                          0, 0);
    }
}

pub fn ungrab_button(context: Context, button: c_uint, modifiers: c_uint, window: Window) {
    unsafe{
        xlib::XUngrabButton(context.display, button, modifiers, window);
    }
}

pub fn get_atom_name(context: Context, atom: xlib::Atom) -> Option<String> {
    unsafe{
        let name = xlib::XGetAtomName(context.display, atom);
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

pub fn get_atom(context: Context, name: &str) -> xlib::Atom{
    unsafe{
        let cstr = ffi::CString::new(name).unwrap();
        let atom = xlib::XInternAtom(context.display, cstr.as_ptr(), xlib::False);
        atom
    }
}

pub fn get_text_property(context: Context, window: xlib::Window, atom: xlib::Atom) -> Option<String>{
    unsafe{
        let mut prop: xlib::XTextProperty = mem::zeroed();
        let r = xlib::XGetTextProperty(context.display, window, &mut prop, atom);
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

pub fn get_wm_protocols(context: Context, window: Window) -> Vec<xlib::Atom>{
    unsafe{
        let mut atoms: *mut xlib::Atom = ptr::null_mut();
        let mut count = 0;
        let s = xlib::XGetWMProtocols(context.display, window, &mut atoms, &mut count);
        let mut vec: Vec<xlib::Atom> = Vec::new();
        if s > 0 {
            vec = slice::from_raw_parts(atoms, count as usize).iter()
                .map(|&a| a as xlib::Atom).collect();
            xlib::XFree(atoms as *mut c_void);
        }
        vec
    }
}

pub fn get_window_attributes(context: Context, window: Window) -> xlib::XWindowAttributes {
    unsafe{
        let mut attrs: xlib::XWindowAttributes = mem::zeroed();
        xlib::XGetWindowAttributes(context.display, window, &mut attrs);
        attrs
    }
}

pub fn get_transient_for_hint(context: Context, window: Window) -> i32 {
    unsafe{
        let mut window_return: xlib::Window = 0;
        xlib::XGetTransientForHint(context.display, window, &mut window_return)
    }
}

pub fn change_window_attributes(context: Context, window: Window, valuemask: c_ulong, attrs: *mut xlib::XSetWindowAttributes) {
    unsafe{
        xlib::XChangeWindowAttributes(context.display, window, valuemask, attrs);
    }

}

fn get_window_property(context: Context, window: Window, atom: xlib::Atom) {
    unsafe {
        let mut actual_type_return: u64 = 0;
        let mut actual_format_return: i32 = 0;
        let mut nitem_return: libc::c_ulong = 0;
        let mut bytes_after_return: libc::c_ulong = 0;
        let mut prop_return: *mut libc::c_uchar = mem::zeroed();

        let r = xlib::XGetWindowProperty(context.display, window, xlib::XA_WM_NAME,
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

pub fn get_top_window(context: Context, window: Window)-> Option<Window>{
    let mut w = window;
    let mut root = window;
    let mut parent = window;
    unsafe{
        while parent != root {
            w = parent;
            unsafe{
                let res = query_tree(context, w);
                match res {
                    Some((r, p, _)) => {
                        parent = p;
                        root = r;
                    }
                    None => {
                        return None
                    }
                }
            }
        }
        Some(w)
    }
}

pub fn get_children(context: Context, window: Window) -> Option<Vec<Window>> {
    let res = query_tree(context, window);
    match res {
        Some((_, _, v)) => {
            Some(v)
        }
        None => { None }
    }
}

pub fn query_tree(context: Context, window: Window) -> Option<(Window, Window, Vec<Window>)> {
    let mut root: Window = 0;
    let mut parent: Window = window;
    let mut children: *mut Window = ptr::null_mut();
    let mut nchildren: libc::c_uint = 0;
    unsafe{
        let s = xlib::XQueryTree(context.display, window, &mut root, &mut parent, &mut children, &mut nchildren);
        if s > 0 {
            let vec = slice::from_raw_parts(children,
                                            nchildren as usize).iter()
                .map(|&a| a as Window).collect();
            xlib::XFree(children as *mut libc::c_void);
            Some((root, parent, vec))
        }
        else{
            None
        }
    }

}

pub fn lookup_keysym(event: xlib::XKeyEvent, index: c_int) -> c_ulong{
    let mut e = event;
    unsafe{
        xlib::XLookupKeysym(&mut e, index)
    }
}

// pub fn loopup_string(event: xlib::XKeyEvent) -> Option<(c_ulong, String)>{
//     let mut sym: xlib::KeySym = 0;
//     let status: *mut xlib::XComposeStatus = ptr::null_mut();

//     unsafe{
//         let mut e = event;
//         xlib::XLookupString(&mut e, ptr::null_mut(), 0, &mut sym, status);
//     }

// }

pub fn set_input_focus(context: Context, window: Window) {
    let none = 0;
    let pointer_root = 1;
    let parent = 2;
    unsafe{
        let (old, _) = get_input_focus(context);
        debug!("set focus from {} to {}", old, window);
        xlib::XSetInputFocus(context.display, window, none, 0);
    }
}

pub fn get_input_focus(context: Context) -> (Window, c_int){
    let mut window: xlib::Window = 0;
    let mut revert_to: libc::c_int = 0;
    unsafe{
        let s = xlib::XGetInputFocus(context.display, &mut window, &mut revert_to);
    }
    (window, revert_to)
}

pub fn unmap_window(context: Context, window: Window) -> c_int{
    unsafe{
        xlib::XUnmapWindow(context.display, window)
    }
}

pub fn configure_window(context: Context, window: Window, mask:c_uint, mut change: xlib::XWindowChanges) {
    unsafe{
        xlib::XConfigureWindow(context.display, window, mask, &mut change);
    }
}
pub fn map_window(context: Context, window: Window) -> c_int{
    unsafe{
        xlib::XMapWindow(context.display, window)
    }
}

pub fn kill_window(context: Context, window: Window) {
    let mut event: xlib::XClientMessageEvent = unsafe {
        mem::zeroed()
    };

    let display = context.display;
    let wm_delete_window = get_atom(context, "WM_DELETE_WINDOW");
    let wm_protocols = get_atom(context, "WM_PROTOCOLS");
    let protocols = get_wm_protocols(context, window);

    if protocols.iter().any(|&p| p == wm_delete_window){
        event.type_ = xlib::ClientMessage;
        event.message_type = wm_protocols;
        event.format = 32;
        event.window = window;
        event.send_event = xlib::True;
        event.display = display;
        event.data.set_long(0, wm_delete_window as libc::c_long);

        let mut e: xlib::XEvent = From::from(event);
        send_event(context, window, xlib::True, xlib::NoEventMask, e);
    }
    else{
        unsafe{
            xlib::XKillClient(display, window);
        }
    }

}

pub fn send_event(context: Context, window: Window,
                  propagate: c_int, event_mask: c_long,
                  mut event: xlib::XEvent) {
    unsafe{
        xlib::XSendEvent(context.display, window, propagate, event_mask, &mut event);
    }
}

pub fn keysym_to_string(keysym: c_ulong) -> Option<String> {
    let mut sym: xlib::KeySym = 0;
    let status: *mut xlib::XComposeStatus = ptr::null_mut();
    unsafe{
        let s = xlib::XKeysymToString(keysym);
        let cstr = ffi::CStr::from_ptr(s);
        match str::from_utf8(cstr.to_bytes()) {
            Ok(s) => {
                Some(s.to_string())
            }
            _ => {
                None
            }
        }
    }
}

pub fn keysym_to_keycode(context: Context, keysym: xlib::KeySym) -> xlib::KeyCode {
    unsafe {
        xlib::XKeysymToKeycode(context.display, keysym)
    }
}

pub fn string_to_keysym(s: &str) -> xlib::KeySym {
    let tmp = ffi::CString::new(s).unwrap();
    unsafe{
        xlib::XStringToKeysym(tmp.as_ptr())
    }

}

pub fn select_input(context: Context, window: Window, mask: c_long) {
    unsafe{
        xlib::XSelectInput(context.display, window, mask);
    }
}

pub fn sync(context: Context, discard: c_int) {
    unsafe {
        xlib::XSync(context.display, discard);
    }
}

pub fn display_height(context: Context, screen_num: c_int) -> c_int{
    unsafe{
        xlib::XDisplayHeight(context.display, screen_num)
    }
}

pub fn display_width(context: Context, screen_num: c_int) -> c_int{
    unsafe{
        xlib::XDisplayWidth(context.display, screen_num)
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
