use std::ptr;

pub struct Node<T> {
    pub data: T,
    pub children: Vec<Node<T>>,
    parent: *mut Node<T>,
}

impl<T> Node<T> {
    pub fn new(data: T) -> Node<T> {
        Node {
            data: data,
            children: Vec::new(),
            parent: ptr::null_mut(),
        }
    }
    pub fn add(&mut self, child: Node<T>) {
        let mut c = child;
        c.parent = self;
        self.children.push(c);
    }

    pub fn insert(&mut self, index: usize, child: Node<T>) {
        let mut c = child;
        c.parent = self;
        self.children.insert(index, c);
    }

    pub fn get_parent(&self) -> Option<&Node<T>> {
        if self.parent == ptr::null_mut() {
            None
        }
        else{
            Some(unsafe{&*self.parent})
        }
    }
}

#[test]
fn nodes() {
    let mut node = Node::<i32>::new(1);
    let mut child = Node::<i32>::new(2);
    node.add(child);
    let p = node.children.get(0).unwrap().get_parent();
    assert!(p.is_some());
    assert_eq!(p.unwrap().data, 1);
}
