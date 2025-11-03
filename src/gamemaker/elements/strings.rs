use crate::gamemaker::deserialize::reader::DataReader;
use crate::gamemaker::deserialize::resources::GMRef;
use crate::gamemaker::elements::{GMChunkElement, GMElement};
use crate::gamemaker::serialize::builder::DataBuilder;
use crate::prelude::*;
use crate::util::assert::assert_int;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

const ALIGNMENT: u32 = 4;

#[derive(Debug, Clone)]
pub struct GMStrings {
    pub strings: Vec<String>,
    pub is_aligned: bool,
    pub exists: bool,
}

impl GMStrings {
    /// Looks up a string in the string table by exact match.
    ///
    /// # Returns
    /// - `Some(GMRef<String>)` if the string exists in the table
    /// - `None` if the string is not found
    ///
    /// # Note
    /// Performs a linear search through all strings.
    pub fn find(&self, target: &str) -> Option<GMRef<String>> {
        self.find_by(|s| s == target)
    }

    /// Finds a string using a custom predicate function.
    ///
    /// # Returns
    /// - `Some(GMRef<String>)` if a string matching the predicate exists in the table
    /// - `None` if a matching string is not found
    ///
    /// # Examples
    /// ```
    /// // Case-insensitive search
    /// table.find_by(|s| s.eq_ignore_ascii_case("HELLO"));
    ///
    /// // Prefix search
    /// table.find_by(|s| s.starts_with("prefix"));
    /// ```
    ///
    /// # Note
    /// Performs a linear search through all strings.
    pub fn find_by<P: Fn(&str) -> bool>(&self, predicate: P) -> Option<GMRef<String>> {
        self.strings
            .iter()
            .enumerate()
            .find(|(_, string)| predicate(string))
            .map(|(i, _)| GMRef::new(i as u32))
    }

    /// Gets or creates a string in the string table.
    ///
    /// If the string already exists in the table, returns the existing reference.
    /// Otherwise, adds the string to the table and returns a new reference.
    ///
    /// # Examples
    /// ```
    /// let ref1 = gm_data.make_string("hello");
    /// let ref2 = gm_data.make_string("hello");
    /// assert_eq!(ref1, ref2); // Same reference for equal strings
    /// ```
    ///
    /// # Note
    /// If you need the string reference to be unique, use [`GMStrings::make_unique`] instead.
    pub fn make(&mut self, target: &str) -> GMRef<String> {
        if let Some(string_ref) = self.find(target) {
            return string_ref;
        }
        self.make_unique(target.to_string())
    }

    /// Adds a new string to the table, guaranteeing uniqueness.
    ///
    /// This is useful for variable and function names, which
    /// use String IDs as identification and thereforce need
    /// unique string references.
    ///
    /// This method always creates a new entry in the string table
    /// without checking for duplicates. The string is assumed to
    /// not already exist in the table.
    ///
    /// # Note
    /// For most use cases, prefer [`make`] which handles deduplication
    /// automatically. Use this method only when you need to force
    /// a new entry or know the string is unique.
    ///
    /// # Examples
    /// ```
    /// // Force adding a duplicate as a separate entry
    /// let ref1 = gm_data.make("hello");
    /// let ref2 = gm_data.make_unique("hello".to_string());
    /// assert_ne!(ref1, ref2); // Different references despite equal content
    /// ```
    pub fn make_unique(&mut self, string: String) -> GMRef<String> {
        let index = self.strings.len();
        self.strings.push(string);
        GMRef::new(index as u32)
    }
}

impl Default for GMStrings {
    fn default() -> Self {
        Self {
            strings: vec![],
            // Align by default for compatibility
            is_aligned: true,
            exists: false,
        }
    }
}

impl Deref for GMStrings {
    type Target = Vec<String>;
    fn deref(&self) -> &Self::Target {
        &self.strings
    }
}

impl DerefMut for GMStrings {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.strings
    }
}

impl GMChunkElement for GMStrings {
    const NAME: &'static str = "STRG";
    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMElement for GMStrings {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let pointers: Vec<u32> = reader.read_simple_list()?;
        let is_aligned: bool = pointers.iter().all(|&p| p % ALIGNMENT == 0);

        let mut strings_by_index: Vec<String> = Vec::with_capacity(pointers.len());
        let mut abs_pos_to_reference: HashMap<u32, GMRef<String>> = HashMap::with_capacity(pointers.len());

        for (i, pointer) in pointers.into_iter().enumerate() {
            reader.cur_pos = pointer;
            if is_aligned {
                reader.align(ALIGNMENT)?;
            }
            let string_length = reader.read_u32()?;
            let string: String = reader.read_literal_string(string_length)?;
            let byte = reader.read_u8()?;
            assert_int("Null terminator byte after string", 0, byte)?;
            strings_by_index.push(string.clone());
            // Occurrence is `start_position + 4` because string refs point to the actual
            // String data instead of the gamemaker element for faster access.
            abs_pos_to_reference.insert(pointer + 4, GMRef::new(i as u32));
        }

        reader.align(0x80)?;
        reader.string_occurrences = abs_pos_to_reference;
        Ok(GMStrings { strings: strings_by_index, is_aligned, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        if !self.exists {
            bail!("Required chunk STRG does not exist");
        }

        builder.write_usize(self.strings.len())?;
        let pointer_list_start: usize = builder.len();
        for _ in 0..self.strings.len() {
            builder.write_u32(0xDEADC0DE);
        }

        for (i, string) in self.strings.iter().enumerate() {
            if self.is_aligned {
                builder.align(ALIGNMENT);
            }
            builder.overwrite_usize(builder.len(), pointer_list_start + 4 * i)?;
            builder.write_usize(string.len())?;
            builder.resolve_pointer(string)?; // Gamemaker string references point to the actual string data
            builder.write_literal_string(string);
            builder.write_u8(0); // Trailing null terminator byte
        }

        builder.align(0x80);
        Ok(())
    }
}

impl GMRef<String> {
    /// Resolves this string reference for display purposes.
    ///
    /// Returns the actual string if the reference is valid, or a placeholder
    /// string (`"<invalid string reference>"`) if the reference is invalid.
    ///
    /// # When to use
    /// - In logging, debugging, or UI contexts where you always want to show something
    /// - In closures or contexts where error propagation is impractical
    ///
    /// # When not to use
    /// - For logic that needs to handle invalid references properly
    /// - When you need to distinguish between valid and invalid references
    ///
    /// **Prefer [`GMRef::resolve`] for proper error handling.**
    ///
    /// # Examples
    /// ```
    /// // Safe for display, even with potentially invalid references
    /// println!("Message: {}", string_ref.display(&gm_data.strings));
    /// ```
    pub fn display<'a>(&self, gm_strings: &'a GMStrings) -> &'a str {
        self.resolve(&gm_strings.strings)
            .map(|i| i.as_str())
            .unwrap_or("<invalid string reference>")
    }
}
