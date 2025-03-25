
pub struct DataChange {
    pub index: usize,
    pub content: Vec<u8>,
    pub delete: bool,
}

impl DataChange {
    pub fn apply(&self, data: Vec<u8>) {
        if self.delete {
            let _ = self.__delete(data);
        } else {
            self.__insert(data)
        }
    }

    fn __insert(&self, mut data: Vec<u8>) {
        data.splice(self.index..self.index, self.content.clone());
    }

    fn __delete(&self, mut data: Vec<u8>) -> Result<(), String> {
        let len: usize = self.content.len();
        if data[self.index..self.index + len] != self.content {
            return Err(format!(
                "Could not delete {} bytes at position {} because they dont exist in the code at the specified location!",
                len, self.index
            ));
        }
        data.splice(self.index..self.index + len, []);
        Ok(())
    }
}
