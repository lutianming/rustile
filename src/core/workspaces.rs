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
            current: '0',
            context: context,
            spaces: HashMap::new(),
        }
    }

    pub fn contain(&self, key: char) -> bool {
        self.spaces.contains_key(&key)
    }

    pub fn create(&mut self, key: char) {
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

    pub fn switch_workspace(&mut self, new: char, context: Context){
        if new == self.current {
            return
        }

        if !self.contain(new) {
            // let s = libx::default_screen(context);
            self.create(new);
        }

        let old = self.current;
        match self.get(old) {
            Some(v) => {
                v.unmap();
                v.update_layout();
            }
            None => {}
        }

        self.current = new;
        match self.get(new) {
            Some(v) => {
                println!("workspace {}", v.id);
                v.map();
                v.focus();
                v.update_layout();
            }
            None => {}
        }
    }

    pub fn can_manage(context: libx::Context, id: Window) -> bool {
        let attrs = libx::get_window_attributes(context, id);
        let transientfor_hint = libx::get_transient_for_hint(context, id);
        attrs.override_redirect == 0 && transientfor_hint == 0
    }

    pub fn move_window(&mut self, window: Window, from: char, to: char){
        if from == to {
            return
        }

        if !self.contain(from) {
            self.create(from);
        }
        if !self.contain(to) {
            self.create(to);
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
        let w = match workspace {
            Some(k) => {
                match self.get(k) {
                    Some(v) => {
                        v
                    }
                    None => {
                        return
                    }
                }
            }
            None => {
                self.current()
            }
        };
        w.add(container);
        w.update_layout();
        w.print_tree(0);
    }

    // insert window just next to old focus
    pub fn insert_window(&mut self, container: Container) {
        match self.get_focus() {
            Some(c) => {
                match c.get_parent(){
                    Some(p) => {
                        let index = p.contain(c.id).unwrap();
                        p.insert(index+1, container);
                        p.update_layout();
                        p.print_tree(0);
                        return
                    }
                    None => {}
                }
            }
            None => {}
        }
        self.add_window(container, None);
    }

    pub fn remove_window(&mut self, window: Window) -> Option<Container>{
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

                    workspace.update_layout();
                    workspace.print_tree(0);
                    return Some(w)
                }
                None => {

                }
            }
        }
        None
    }

    pub fn set_focus(&mut self, window: Window) {
        match self.get_focus() {
            Some(w) => {
                w.unfocus();
            }
            None => {}
        }

        match self.get_container(window) {
            Some((_, c)) => {
                c.focus()
            }
            None => {}
        }
    }

    pub fn get_focus(&mut self) -> Option<&mut Container> {
        let (w, _) = libx::get_input_focus(self.context);
        let res = self.get_container(w);
        match res {
            Some((k, c)) => {
                Some(c)
            }
            None => { None }
        }
    }

    pub fn get_container(&mut self, id: Window) -> Option<(char, &mut Container)>{
        for (k, w) in self.spaces.iter_mut() {
            let r = w.tree_search(id);
            if r.is_some(){
                return Some((*k, r.unwrap()))
            }
        }
        None
    }

    // pub fn find_window(&self, window: Window) -> Option<(char, usize)> {
    //     for (k, w) in self.spaces.iter() {
    //         let index = w.contain(window);
    //         match index {
    //             Some(i) => {
    //                 return Some((*k, i))
    //             }
    //             None => {}
    //         }
    //     }
    //     None
    // }
}
