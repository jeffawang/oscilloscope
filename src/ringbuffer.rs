pub struct RingBuffer<T> {
    pub data: Vec<T>,
    start: usize,
}

impl<T: Copy> RingBuffer<T> {
    pub fn new(data: Vec<T>) -> Self {
        let end = data.len() - 1;
        Self { data, start: 0 }
    }

    pub fn push(&mut self, item: T) {
        self.data[self.start] = item;
        self.start = (self.start + 1) % self.data.len();
    }

    pub fn iter(&mut self) -> Iter<T> {
        Iter {
            data: &self.data,
            curr: self.start,
            returned: 0,
        }
    }
}

pub struct Iter<'a, T> {
    data: &'a Vec<T>,
    curr: usize,
    returned: usize,
}

impl<T: Copy> Iterator for Iter<'_, T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.returned == self.data.len() {
            None
        } else {
            self.curr = (self.curr + 1) % self.data.len();
            self.returned += 1;
            Some(self.data[self.curr])
        }
    }
}
