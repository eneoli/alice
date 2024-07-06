pub struct Counter {
    count: usize,
}

impl Counter {
    pub fn new() -> Self {
        Self { count: 0 }
    }

    pub fn next(&mut self) -> usize {
        let result = self.count;
        self.count += 1;

        result
    }
}
