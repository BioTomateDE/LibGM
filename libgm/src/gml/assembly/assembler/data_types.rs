use std::ops::Index;

use crate::{gml::instruction::DataType, prelude::*};

#[derive(Debug, Clone, Copy)]
pub struct DataTypes(Option<DataType>, Option<DataType>);

impl DataTypes {
    #[must_use]
    pub const fn new() -> Self {
        Self(None, None)
    }

    #[must_use]
    const fn count(self) -> u8 {
        if self.1.is_some() {
            return 2;
        }
        if self.0.is_some() {
            return 1;
        }
        0
    }

    pub fn assert_count(self, count: u8, mnemonic: &str) -> Result<()> {
        let actual = self.count();
        if actual != count {
            bail!(
                "Expected {count} data types for {mnemonic:?} instruction, got {actual} data types"
            );
        }
        Ok(())
    }

    pub fn push(&mut self, data_type: DataType) -> Result<()> {
        if self.1.is_some() {
            bail!("An Instruction can only have 0-2 data types");
        }
        if self.0.is_some() {
            self.1 = Some(data_type);
        } else {
            self.0 = Some(data_type);
        }
        Ok(())
    }
}

/// Indexing is only meant to be used after validating the type count.
impl Index<u8> for DataTypes {
    type Output = DataType;
    fn index(&self, index: u8) -> &Self::Output {
        match index {
            0 => self.0.as_ref().unwrap(),
            1 => self.1.as_ref().unwrap(),
            _ => unreachable!("Invalid Index"),
        }
    }
}

impl DataType {
    pub(super) fn from_char(data_type: char) -> Result<Self> {
        Ok(match data_type {
            'v' => Self::Variable,
            'i' => Self::Int32,
            's' => Self::String,
            'e' => Self::Int16,
            'd' => Self::Double,
            'l' => Self::Int64,
            'b' => Self::Boolean,
            _ => bail!("Invalid data type '{data_type}'"),
        })
    }
}
