extern crate x11;
extern crate libc;

use std::collections::HashMap;
use super::Window;
use super::Workspace;
use super::super::libx;
use super::super::libx::{ Context };


pub struct Workspaces {
    current: char,
    pub spaces: HashMap<char, Workspace>,
}

impl Workspaces {
    pub fn new() -> Workspaces {
        Workspaces {
            current: '1',
            spaces: HashMap::new(),
        }
    }

    pub fn contain(&self, key: char) -> bool {
        self.spaces.contains_key(&key)
    }

    pub fn create(&mut self, key: char, screen_num: libc::c_int) {
        let space = Workspace::new(screen_num);
        self.spaces.insert(key, space);
    }

    pub fn delete(&mut self, key: char) {
        self.spaces.remove(&key);
    }

    pub fn get(&mut self, key: char) -> Option<&mut Workspace>{
        self.spaces.get_mut(&key)
    }

    pub fn current(&mut self) -> &mut Workspace {
        self.spaces.get_mut(&self.current).unwrap()
    }

    pub fn current_name(&self) -> char {
        self.current
    }

    pub fn switch_current(&mut self, new: char, context: Context){
        if new == self.current {
            return
        }

        let old = self.current;
        self.current = new;

        if !self.contain(old) {
            let s = libx::default_screen(context);
            self.create(old, s);
        }
        if !self.contain(new) {
            let s = libx::default_screen(context);
            self.create(new, s);
        }

        match self.get(old) {
            Some(v) => {
                v.hide();
                v.config(context);
            }
            None => {}
        }

        match self.get(new) {
            Some(v) => {
                v.show(context);
                v.config(context);
            }
            None => {}
        }
    }

    pub fn move_window(&mut self, window: Window, from: char, to: char, context: Context){
        if from == to {
            return
        }

        if !self.contain(from) {
            let s = libx::default_screen(context);
            self.create(from, s);
        }
        if !self.contain(to) {
            let s = libx::default_screen(context);
            self.create(to, s);
        }

        match self.get(from) {
            Some(w) => { w.remove(window, context);}
            None => {}
        }

        match self.get(to) {
            Some(w) => { w.add(window, context);}
            None => {}
        }
    }

    pub fn add_window(&mut self, window: Window, workspace: Option<char>, context: Context) {
        match workspace {
            Some(k) => {
                match self.get(k) {
                    Some(w) => {
                        w.add(window, context);
                    }
                    None => {}
                }
            }
            None => {
                // use current workspace
                let c = self.current();
                c.add(window, context);
            }
        }
        debug!("add window {}", window.id);
    }

    pub fn remove_window(&mut self, window: Window, context: Context) {
        // let c = self.current();
        // if c.remove(window, context) {
        //     return
        // }

        for (k, w) in self.spaces.iter_mut() {
            if w.remove(window, context){
                return;
            }
        }
    }

    pub fn set_focus(&mut self, window: Window, context: Context) {
        let res = self.get_focus(context);
        match res {
            Some(w) => {
                w.unfocus();
            }
            None => {}
        }

        let res = self.find_window(window);
        match res {
            Some((k, index)) => {
                self.get(k).unwrap().get(index).focus();
            }
            None => {}
        }
    }

    pub fn get_focus(&mut self, context: Context) -> Option<Window> {
        let (w, _) = libx::get_input_focus(context);
        let res = self.find_window(Window::new(context, w));
        match res {
            Some((k, index)) => {
                Some(self.get(k).unwrap().get(index))
            }
            None => { None }
        }
    }

    pub fn find_window(&self, window: Window) -> Option<(char, usize)> {
        for (k, w) in self.spaces.iter() {
            let index = w.contain(window);
            match index {
                Some(i) => {
                    return Some((*k, i))
                }
                None => {}
            }
        }
        None
    }
}
