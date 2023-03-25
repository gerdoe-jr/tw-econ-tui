#[derive(Debug, Clone, Copy)]
pub struct StringArray<const MAX_LENGTH: usize> {
    pub len: usize,
    pub array: [char; MAX_LENGTH]
}

impl<const MAX_LENGTH: usize> StringArray<MAX_LENGTH> {
    pub fn new() -> Self {
        Self { len: 0, array: ['0'; MAX_LENGTH] }
    }

    pub fn push(&mut self, c: char) {
        if self.len < MAX_LENGTH {
            self.array[self.len] = c;
            self.len += 1;
        }
    }

    pub fn pop(&mut self) {
        if self.len > 0 {
            self.len -= 1;
        }
    }
}

impl<const MAX_LENGTH: usize> ToString for StringArray<MAX_LENGTH> {
    fn to_string(&self) -> String {
        let result: String = self.array[..self.len].iter().collect();

        result
    }
}
