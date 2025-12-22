use crate::{
    gamemaker::{
        deserialize::reader::DataReader, elements::GMElement, serialize::builder::DataBuilder,
    },
    prelude::*,
    util::init::vec_with_capacity,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GMCodeLocals {
    pub code_locals: Vec<GMCodeLocal>,
    pub exists: bool,
}

impl GMElement for GMCodeLocals {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let code_locals: Vec<GMCodeLocal> = reader.read_simple_list()?;
        Ok(Self { code_locals, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_simple_list(&self.code_locals)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GMCodeLocal {
    pub name: String,
    pub variables: Vec<LocalVariable>,
}

impl GMElement for GMCodeLocal {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let local_variables_count = reader.read_u32()?;
        let name: String = reader.read_gm_string()?;
        let mut variables: Vec<LocalVariable> = vec_with_capacity(local_variables_count)?;
        for _ in 0..local_variables_count {
            variables.push(LocalVariable::deserialize(reader)?);
        }
        Ok(Self { name, variables })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_usize(self.variables.len())?;
        builder.write_gm_string(&self.name);
        for local_var in &self.variables {
            local_var.serialize(builder)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalVariable {
    /// unknown what this does
    pub weird_index: u32,
    pub name: String,
}

impl GMElement for LocalVariable {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let weird_index = reader.read_u32()?;
        let name: String = reader.read_gm_string()?;
        Ok(Self { weird_index, name })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_u32(self.weird_index);
        builder.write_gm_string(&self.name);
        Ok(())
    }
}
