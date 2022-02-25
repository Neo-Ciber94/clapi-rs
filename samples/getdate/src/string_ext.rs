pub trait StringExt {
    fn pad_left(self, len: usize, char: char) -> String;
    fn pad_right(self, len: usize, char: char) -> String;
}

impl StringExt for String {
    fn pad_left(mut self, len: usize, char: char) -> String {
        let count = len.checked_sub(self.len()).unwrap_or(0);
        for _ in 0..count {
            self.insert(0, char);
        }

        self
    }

    fn pad_right(mut self, len: usize, char: char) -> String {
        let count = len.checked_sub(self.len()).unwrap_or(0);
        for _ in 0..count {
            self.push(char);
        }

        self
    }
}
