use std::ops::{Deref, DerefMut};

use crate::{
    gamemaker::{
        deserialize::{reader::DataReader, resources::resource_opt_from_i32},
        elements::{GMChunkElement, GMElement},
        reference::GMRef,
        serialize::builder::DataBuilder,
    },
    gml::instructions::GMCode,
    prelude::*,
};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GMScripts {
    pub scripts: Vec<GMScript>,
    pub exists: bool,
}

impl GMScripts {
    fn index_by_name(&self, name: &str) -> Result<usize> {
        for (i, script) in self.scripts.iter().enumerate() {
            if script.name == name {
                return Ok(i);
            }
        }

        bail!("Could not find script with name {name:?}");
    }

    pub fn ref_by_name(&self, name: &str) -> Result<GMRef<GMScript>> {
        self.index_by_name(name).map(GMRef::from)
    }

    pub fn by_name(&self, name: &str) -> Result<&GMScript> {
        self.index_by_name(name).map(|index| &self.scripts[index])
    }

    pub fn by_name_mut(&mut self, name: &str) -> Result<&mut GMScript> {
        self.index_by_name(name)
            .map(|index| &mut self.scripts[index])
    }
}

impl Deref for GMScripts {
    type Target = Vec<GMScript>;
    fn deref(&self) -> &Self::Target {
        &self.scripts
    }
}

impl DerefMut for GMScripts {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.scripts
    }
}

impl GMChunkElement for GMScripts {
    const NAME: &'static str = "SCPT";
    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMElement for GMScripts {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let scripts: Vec<GMScript> = reader.read_pointer_list()?;
        Ok(Self { scripts, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_pointer_list(&self.scripts)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMScript {
    pub name: String,
    pub is_constructor: bool,
    pub code: Option<GMRef<GMCode>>,
}

impl GMElement for GMScript {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: String = reader.read_gm_string()?;
        let mut code_id: i32 = reader.read_i32()?;
        let mut is_constructor: bool = false;
        if code_id < -1 {
            code_id &= 0x7FFF_FFFF;
            is_constructor = true;
        }
        let code: Option<GMRef<GMCode>> = resource_opt_from_i32(code_id)?;
        Ok(GMScript { name, is_constructor, code })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.name);
        if self.is_constructor {
            if let Some(gm_code_ref) = &self.code {
                builder.write_u32(gm_code_ref.index | 0x8000_0000);
            } else {
                builder.write_i32(-1);
            }
        } else {
            builder.write_resource_id_opt(&self.code);
        }
        Ok(())
    }
}
