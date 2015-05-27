extern crate x11;
extern crate libc;

use std::collections::HashMap;
// use super::Window;
use x11::xlib::{Window};
use super::Container;
use super::super::libx;
use super::super::libx::{ Context };


pub struct Workspaces {
    current: char,
    context: Context,
    pub spaces: HashMap<char, Container>,
}

impl Workspaces {
    pub fn new(context: Context) -> Workspaces {
        Workspaces {
            current: '1',
            context: context,
            spaces: HashMap::new(),
        }
    }

    pub fn contain(&self, key: char) -> bool {
        self.spaces.contains_key(&key)
    }

    pub fn create(&mut self, key: char, screen_num: libc::c_int) {
        let space = Container::from_id(self.context, self.context.root);
        self.spaces.insert(key, space);
    }

    pub fn delete(&mut self, key: char) {
        self.spaces.remove(&key);
    }

    pub fn get(&mut self, key: char) -> Option<&mut Container>{
        self.spaces.get_mut(&key)
    }

    pub fn current(&mut self) -> &mut Container {
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
                v.unmap();
                v.update();
            }
            None => {}
        }

        match self.get(new) {
            Some(v) => {
                v.map();
                v.update();
            }
            None => {}
        }
    }

    pub fn can_manage(context: libx::Context, id: Window) -> bool {
        let attrs = libx::get_window_attributes(context, id);
        let transientfor_hint = libx::get_transient_for_hint(context, id);
        attrs.override_redirect == 0 && transientfor_hint == 0
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

        let res = match self.get(from) {
            Some(w) => { w.remove(window) }
            None => { None }
        };

        match self.get(to) {
            Some(w) => {
                if res.is_some(){
                    w.add(res.unwrap());
                }
            }
            None => {}
        }
    }

    pub fn add_window(&mut self, container: Container, workspace: Option<char>) {
        match workspace {
            Some(k) => {
                match self.get(k) {
                    Some(w) => {
                        w.add(container);
                        w.update();
                    }
                    None => {}
                }
            }
            None => {
                // use current workspace
                let current = self.current();
                current.add(container);
                current.update();
                current.print_tree(0);
            }
        }
    }

    pub fn remove_window(&mut self, window: Window, context: Context) {
        for (k, workspace) in self.spaces.iter_mut() {
            let res =  workspace.remove(window);
            match res {
                Some(w) => {
                    if w.pid.is_some() {
                        let res = workspace.remove(w.pid.unwrap());
                        if res.is_some(){
                            res.unwrap().destroy();
                        };
                    }

                    workspace.update();
                    workspace.print_tree(0);
                    return;
                }
                None => {

                }
            }
        }
    }

    pub fn set_focus(&mut self, window: Window, context: Context) {
        match self.get_focus(context) {
            Some(w) => {
                w.unfocus();
            }
            None => {}
        }

        match self.get_container(window) {
            Some(c) => {
                c.focus()
            }
            None => {}
        }
    }

    pub fn get_focus(&mut self, context: Context) -> Option<&Container> {
        let (w, _) = libx::get_input_focus(context);
        let res = self.find_window(w);
        match res {
            Some((k, index)) => {
                self.get(k).unwrap().get(index)
            }
            None => { None }
        }
    }

    pub fn get_container(&self, id: Window) -> Option<&Container>{
        for (k, w) in self.spaces.iter() {
            let index = w.contain(id);
            match index {
                Some(i) => {
                    return w.get(i)
                }
                None => {}
            }
        }
        None
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
