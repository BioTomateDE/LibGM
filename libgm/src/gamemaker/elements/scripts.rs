use crate::gamemaker::deserialize::reader::DataReader;
use crate::gamemaker::deserialize::resources::GMRef;
use crate::gamemaker::deserialize::resources::resource_opt_from_i32;
use crate::gamemaker::elements::code::GMCode;
use crate::gamemaker::elements::{GMChunkElement, GMElement};
use crate::gamemaker::serialize::builder::DataBuilder;
use crate::prelude::*;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Default)]
pub struct GMScripts {
    pub scripts: Vec<GMScript>,
    pub exists: bool,
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
    pub name: GMRef<String>,
    pub is_constructor: bool,
    pub code: Option<GMRef<GMCode>>,
}

impl GMElement for GMScript {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let mut code_id: i32 = reader.read_i32()?;
        let mut is_constructor: bool = false;
        if code_id < -1 {
            code_id &= 0x7FFFFFFF;
            is_constructor = true;
        }
        let code: Option<GMRef<GMCode>> = resource_opt_from_i32(code_id)?;
        Ok(GMScript { name, is_constructor, code })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.name)?;
        if self.is_constructor {
            if let Some(gm_code_ref) = &self.code {
                builder.write_u32(gm_code_ref.index | 0x80000000);
            } else {
                builder.write_i32(-1);
            }
        } else {
            builder.write_resource_id_opt(&self.code);
        }
        Ok(())
    }
}
