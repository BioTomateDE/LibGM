// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::util::init::vec_with_capacity;
use crate::wad::elem::GMElement;
use crate::wad::load::reader::DataReader;
use crate::wad::save::writer::DataWriter;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeLocal {
    pub name: GMRef<String>,
    pub variables: Vec<LocalVariable>,
}

impl GMElement for CodeLocal {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let local_variables_count = reader.read_u32()?;
        let name: GMRef<String> = reader.read_gm_string()?;
        let mut variables: Vec<LocalVariable> = vec_with_capacity(local_variables_count)?;
        for _ in 0..local_variables_count {
            variables.push(LocalVariable::deserialize(reader)?);
        }
        Ok(Self { name, variables })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_usize(self.variables.len())?;
        writer.write_gm_string(self.name)?;
        for local_var in &self.variables {
            writer.write_elem(local_var)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalVariable {
    /// unknown what this does
    pub weird_index: u32,
    pub name: GMRef<String>,
}

impl GMElement for LocalVariable {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let weird_index = reader.read_u32()?;
        let name: GMRef<String> = reader.read_gm_string()?;
        Ok(Self { weird_index, name })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_u32(self.weird_index);
        writer.write_gm_string(self.name)?;
        Ok(())
    }
}
