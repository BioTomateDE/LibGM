use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{GMChunkElement, GMElement},
        serialize::builder::DataBuilder,
    },
    prelude::*,
    util::assert::assert_int,
};

const ALIGNMENT: u32 = 4;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GMStrings;

impl GMChunkElement for GMStrings {
    const NAME: &'static str = "STRG";
    fn exists(&self) -> bool {
        true
    }
}

impl GMElement for GMStrings {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let pointers: Vec<u32> = reader.read_simple_list()?;
        let is_aligned: bool = pointers.iter().all(|&p| p.is_multiple_of(ALIGNMENT));

        let mut strings: Vec<String> = Vec::with_capacity(pointers.len());

        for pointer in pointers {
            if is_aligned {
                reader.align(ALIGNMENT)?;
            }
            reader.assert_pos(pointer, "String")?;
            let string_length = reader.read_u32()?;
            let string: String = reader.read_literal_string(string_length)?;
            let byte = reader.read_u8()?;
            assert_int("Null terminator byte after string", 0, byte)?;
            strings.push(string.clone());
        }

        reader.align(0x80)?;
        reader.strings = strings;
        Ok(Self)
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        let mut strings = std::mem::take(&mut builder.string_placeholders);
        strings.sort_unstable_by(|a, b| a.string.cmp(&b.string));
        let count = count_unique_strings(&strings);

        // Prepare Pointer List
        builder.write_usize(count)?;
        let pointer_list_start: usize = builder.len();
        for _ in 0..count {
            builder.write_u32(0xDEAD_C0DE);
        }

        let mut string = None;
        let mut string_position = 0xDEAD_C0DE;
        let mut index = 0;

        for placeholder in strings {
            // For identical strings, just write the the position/id again (since they're sorted).
            if string.as_ref() == Some(&placeholder.string) {
                overwrite_placeholder(builder, &placeholder, string_position, index - 1)?;
                continue;
            }

            // Write String

            builder.align(ALIGNMENT);

            let list_pointer = builder.len();
            let list_placeholder_pos = pointer_list_start + index * 4;
            builder.overwrite_usize(list_pointer, list_placeholder_pos)?;

            builder.write_usize(placeholder.string.len())?;

            string_position = builder.len();
            builder.write_literal_string(&placeholder.string);

            // Trailing null terminator byte
            builder.write_u8(0);

            overwrite_placeholder(builder, &placeholder, string_position, index)?;
            index += 1;
            string = Some(placeholder.string);
        }

        builder.align(0x80);
        Ok(())
    }
}

/// Used when building data file
#[derive(Debug)]
pub struct StringPlaceholder {
    pub placeholder_position: u32,
    pub string: String,

    /// Whether to write the String ID/Index instead of a pointer.
    pub write_id: bool,
}

fn count_unique_strings(sorted_vec: &[StringPlaceholder]) -> usize {
    if sorted_vec.is_empty() {
        return 0;
    }

    let mut count = 1;
    let mut prev = &sorted_vec[0].string;

    for item in &sorted_vec[1..] {
        if &item.string != prev {
            count += 1;
            prev = &item.string;
        }
    }

    count
}

fn overwrite_placeholder(
    builder: &mut DataBuilder,
    placeholder: &StringPlaceholder,
    string_position: usize,
    index: usize,
) -> Result<()> {
    let placeholder_position = placeholder.placeholder_position as usize;
    let number = if placeholder.write_id {
        index
    } else {
        string_position
    };
    builder.overwrite_usize(number, placeholder_position)?;
    Ok(())
}
