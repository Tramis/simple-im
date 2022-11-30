//! alloc id

#[derive(Debug)]
pub struct Allocator {
    last: usize,
    freelist: Vec<usize>,
}

impl Allocator {
    pub fn new() -> Self {
        Self {
            last: 0,
            freelist: vec![],
        }
    }

    pub fn get(&mut self) -> usize {
        match self.freelist.pop() {
            Some(v) => v,
            None => {
                self.last += 1;
                self.last
            }
        }
    }

    pub fn len(&self) -> usize {
        self.last - self.freelist.len()
    }

    pub fn remove(&mut self, id: usize) {
        assert!(self.last >= id);
        self.freelist.push(id)
    }
}
