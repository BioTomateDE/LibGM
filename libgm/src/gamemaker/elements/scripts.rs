use macros::list_chunk;

use crate::{
    gamemaker::{
        deserialize::{reader::DataReader, resources::resource_opt_from_i32},
        elements::GMElement,
        reference::GMRef,
        serialize::builder::DataBuilder,
    },
    gml::instructions::GMCode,
    prelude::*,
};

#[list_chunk("SCPT")]
pub struct GMScripts {
    pub scripts: Vec<GMScript>,
    pub exists: bool,
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
        Ok(Self { name, is_constructor, code })
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
            builder.write_resource_id_opt(self.code);
        }
        Ok(())
    }
}
