use crate::deserialize::strings::GMStrings;


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum RefKind {
    String,
    TexturePage,
    TexturePageData,
    Texture,
    Sprite,
    Audio,
    Sound,
    Function,
    Script,
    GameObject,
    Font,
    Background,
}


// stores actual referenced kind/datatype in `.kind`, to make mixing possible.
// also has a fake type generic so that the kind/datatype can be specified in struct fields for clarity
// if kind/datatype is mixed; use () as generic T
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct GMRef<T> {
    pub index: usize,
    kind: RefKind,
    _marker: std::marker::PhantomData<T>,
}
impl<T> GMRef<T> {
    pub fn new(index: usize, kind: RefKind) -> GMRef<T> {
        Self {
            index,
            kind,
            _marker: std::marker::PhantomData,
        }
    }
    pub fn string(index: usize) -> Self {
        Self::new(index, RefKind::String)
    }
    pub fn texture_page(index: usize) -> Self {
        Self::new(index, RefKind::TexturePage)
    }
    pub fn texture_page_data(index: usize) -> Self {
        Self::new(index, RefKind::TexturePageData)
    }
    pub fn texture(index: usize) -> Self {
        Self::new(index, RefKind::Texture)
    }
    pub fn sprite(index: usize) -> Self {
        Self::new(index, RefKind::Sprite)
    }
    pub fn audio(index: usize) -> Self {
        Self::new(index, RefKind::Audio)
    }
    pub fn sound(index: usize) -> Self {
        Self::new(index, RefKind::Sound)
    }
    pub fn function(index: usize) -> Self {
        Self::new(index, RefKind::Function)
    }
    pub fn script(index: usize) -> Self {
        Self::new(index, RefKind::Script)
    }
    pub fn game_object(index: usize) -> Self {
        Self::new(index, RefKind::GameObject)
    }
    pub fn font(index: usize) -> Self {
        Self::new(index, RefKind::Font)
    }
    pub fn background(index: usize) -> Self {
        Self::new(index, RefKind::Background)
    }
}
impl<'a, T> GMRef<T> {
    pub fn resolve(&self, elements_by_index: &'a Vec<T>) -> Result<&'a T, String> {
        elements_by_index.get(self.index)
            .ok_or(format!(
                "Could not resolve {} reference with \
                index {} in list with length {}.",
                std::any::type_name::<T>(),
                self.index,
                elements_by_index.len(),
            ))
    }
}


#[derive(Debug, Clone)]
pub struct GMChunk<'a> {
    pub name: String,       // 4 letter name of chunk
    pub abs_pos: usize,     // absolute position/index in data.win file
    pub data: &'a [u8],     // raw data
    pub file_index: usize,  // gets incremented by .read_{} methods when parsing chunk
}

impl GMChunk<'_> {
    // fn apply_changes(&self, mut changes: Vec<DataChange>) {
    //     changes.sort_by(|a, b| b.index.cmp(&a.index));
    //     for change in changes {
    //         println!(
    //             "[DataChange @ {}] Index: {} | Len: {} | Delete: {}",
    //             self.name,
    //             change.index,
    //             change.content.len(),
    //             change.delete
    //         );
    //         change.apply(self.data.clone());
    //     }
    // }


    pub fn read_u64(&mut self) -> Result<u64, String> {
        let bytes = self.data
            .get(self.file_index..self.file_index + 8)
            .ok_or(format!(
                "Trying to read u64 out of bounds in chunk '{}' at position {}: {} > {}.",
                self.name,
                self.file_index,
                self.file_index + 8,
                self.data.len(),
            ))?;
        self.file_index += 8;
        Ok(u64::from_le_bytes(bytes.try_into().unwrap()))
    }
    
    pub fn read_i64(&mut self) -> Result<i64, String> {
        let bytes = self.data
            .get(self.file_index..self.file_index + 8)
            .ok_or(format!(
                "Trying to read i64 out of bounds in chunk '{}' at position {}: {} > {}.",
                self.name,
                self.file_index,
                self.file_index + 8,
                self.data.len(),
            ))?;
        self.file_index += 8;
        Ok(i64::from_le_bytes(bytes.try_into().unwrap()))
    }
    
    pub fn read_u32(&mut self) -> Result<u32, String> {
        let bytes = self.data
            .get(self.file_index..self.file_index + 4)
            .ok_or(format!(
                "Trying to read u32 out of bounds in chunk '{}' at position {}: {} > {}.",
                self.name,
                self.file_index,
                self.file_index + 4,
                self.data.len(),
            ))?;
        self.file_index += 4;
        Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
    }
    
    pub fn read_i32(&mut self) -> Result<i32, String> {
        let bytes = self.data
            .get(self.file_index..self.file_index + 4)
            .ok_or(format!(
                "Trying to read i32 out of bounds in chunk '{}' at position {}: {} > {}.",
                self.name,
                self.file_index,
                self.file_index + 4,
                self.data.len(),
            ))?;
        self.file_index += 4;
        Ok(i32::from_le_bytes(bytes.try_into().unwrap()))
    }
    
    pub fn read_u16(&mut self) -> Result<u16, String> {
        let bytes = self.data
            .get(self.file_index..self.file_index + 2)
            .ok_or(format!(
                "Trying to read u16 out of bounds in chunk '{}' at position {}: {} > {}.",
                self.name,
                self.file_index,
                self.file_index + 2,
                self.data.len(),
            ))?;
        self.file_index += 2;
        Ok(u16::from_le_bytes(bytes.try_into().unwrap()))
    }
    
    pub fn read_i16(&mut self) -> Result<i16, String> {
        let bytes = self.data
            .get(self.file_index..self.file_index + 2)
            .ok_or(format!(
                "Trying to read i16 out of bounds in chunk '{}' at position {}: {} > {}.",
                self.name,
                self.file_index,
                self.file_index + 2,
                self.data.len(),
            ))?;
        self.file_index += 2;
        Ok(i16::from_le_bytes(bytes.try_into().unwrap()))
    }
    
    pub fn read_u8(&mut self) -> Result<u8, String> {
        let byte = *self.data
            .get(self.file_index)
            .ok_or(format!(
                "Trying to read u8 out of bounds in chunk '{}' at position {}.",
                self.name,
                self.file_index,
            ))?;
        self.file_index += 1;
        Ok(byte)
    }
    
    pub fn read_i8(&mut self) -> Result<i8, String> {
        let byte = *self.data
            .get(self.file_index)
            .ok_or(format!(
                "Trying to read i8 out of bounds in chunk '{}' at position {}.",
                self.name,
                self.file_index,
            ))?;
        self.file_index += 1;
        Ok(byte as i8)
    }

    pub fn read_usize(&mut self) -> Result<usize, String> {
        // Read unsigned 32-bit integer and convert to usize (little endian)
        static FAILSAFE_AMOUNT: usize = 100_000_000;
        let number: u32 = self.read_u32()?;
        let number: usize = number as usize;

        if number < FAILSAFE_AMOUNT {
            return Ok(number)
        }
        Err(format!(
            "Failsafe triggered in chunk '{}' at position {} trying \
            to read usize integer: Number {} is larger than failsafe amount {}.",
            self.name,
            self.file_index - 4,
            number,
            FAILSAFE_AMOUNT
        ))
    }

    pub fn read_usize_big_endian(&mut self, enable_failsafe: bool) -> Result<usize, String> {
        // Read unsigned 32-bit integer and convert to usize (big endian)
        static FAILSAFE_AMOUNT: usize = 200_000_000;

        let bytes: [u8; 4] = self.data.get(self.file_index .. self.file_index + 4)
            .ok_or(format!(
                "Trying to read big endian usize integer (u32) \
                out of bounds in chunk '{}' at position {}: {} > {}.", 
                self.name,
                self.file_index,
                self.file_index + 4,
                self.data.len()
            ))?
            .try_into().unwrap();
        self.file_index += 4;

        let number: u32 = u32::from_be_bytes(bytes);
        let number: usize = number as usize;

        if number < FAILSAFE_AMOUNT || !enable_failsafe {
            return Ok(number);
        }
        Err(format!(
            "Failsafe triggered in chunk '{}' at position {} trying \
            to read big endian usize integer: Number {} is larger than failsafe amount {}.",
            self.name,
            self.file_index - 4,
            number,
            FAILSAFE_AMOUNT
        ))
    }

    pub fn read_f32(&mut self) -> Result<f32, String> {
        // Read a single-precision floating point number (little endian)
        if self.file_index + 4 > self.data.len() {
            return Err(format!(
                "Trying to read f32 out of bounds in chunk '{}' at position {}: {} > {}.",
                self.name,
                self.file_index,
                self.file_index + 4,
                self.data.len()
            ));
        }

        let raw: [u8; 4] = self.data[self.file_index .. self.file_index + 4].try_into().unwrap();
        let number: f32 = f32::from_le_bytes(raw);
        self.file_index += 4;
        Ok(number)
    }


    pub fn read_literal_string(&mut self, length: usize) -> Result<String, String> {
        // Read literal ascii/utf8 string with specified length
        let bytes: Vec<u8> = match self.data.get(self.file_index..self.file_index + length) {
            Some(bytes) => bytes.to_owned(),
            None => return Err(format!(
                "Trying to read literal string with length {} out of bounds \
                in chunk '{}' at position {}: {} > {}.",
                length, self.name, self.file_index, self.file_index + length, self.data.len()
            )),
        };
        self.file_index += length;

        let string = match String::from_utf8(bytes) {
            Ok(string) => string,
            Err(error) => {
                return Err(format!(
                    "Could not parse literal string with length {} in chunk '{}' at position {}: {}",
                    length,
                    self.name,
                    self.file_index - length,
                    error
                ));
            }
        };
        Ok(string)
    }

    pub fn read_chunk_name(&mut self) -> Result<String, String> {
        // Read chunk name (4 ascii characters)
        if self.file_index + 4 > self.data.len() {
            return Err(format!(
                "Trying to read chunk name out of bounds at position {}: {} > {}.",
                self.file_index,
                self.file_index + 4,
                self.data.len()
            ));
        }

        match self.read_literal_string(4) {
            Ok(string) => Ok(string),
            Err(error) => Err(format!("Could not parse chunk name at position {}: {}", self.file_index, error))
        }
    }

    pub fn read_gm_string(&mut self, gm_strings: &GMStrings) -> Result<GMRef<String>, String> {
        let string_abs_pos: usize = self.read_usize()?;
        let string_ref = gm_strings.abs_pos_to_reference.get(&string_abs_pos)
            .ok_or(format!(
                "Could not read reference string with absolute position {} in chunk '{}' at \
                position {} because it doesn't exist in the string map (length: {}).",
                string_abs_pos,
                self.name,
                self.file_index - 4,
                gm_strings.abs_pos_to_reference.len(),
            ))?;
        Ok(string_ref.clone())
    }

    pub fn read_pointer_list(&mut self) -> Result<Vec<usize>, String> {
        let pointer_position: usize = match self.read_usize()?.checked_sub(self.abs_pos) {
            Some(pos) => pos,
            None => return Err(format!(
                "Start of Pointer list underflowed at position {} in chunk '{}' with absolute position {}.",
                self.file_index - 4, self.name, self.abs_pos,
        ))};

        let old_position: usize = self.file_index;
        self.file_index = pointer_position;

        let pointer_count: usize = self.read_usize()?;
        let mut pointers: Vec<usize> = Vec::with_capacity(pointer_count);
        for _ in 0..pointer_count {
            let pointer: usize = match self.read_usize()?.checked_sub(self.abs_pos) {
                Some(pos) => pos,
                None => return Err(format!(
                    "Element of Pointer list underflowed at position {} in chunk '{}' with absolute position {}.",
                    self.file_index - 4, self.name, self.abs_pos,
                ))};
            pointers.push(pointer);
        }

        self.file_index = old_position;
        Ok(pointers)
    }
}
