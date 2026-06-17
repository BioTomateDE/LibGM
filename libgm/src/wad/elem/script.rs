// SPDX-License-Identifier: GPL-3.0-only

use crate::gml::Code;
use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::chunk::gm_named_list_chunk;
use crate::wad::elem::GMElement;
use crate::wad::parse::reader::DataReader;
use crate::wad::reference::GMRef;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Scripts {
    pub elems: Vec<Script>,
    pub exists: bool,
}

// not sure if direct
gm_named_list_chunk!(SCPT, Scripts, Script, direct);

impl GMElement for Scripts {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let elems: Vec<Script> = reader.read_pointer_list()?;
        Ok(Self { elems, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_pointer_list(&self.elems)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Script {
    pub name: GMRef<String>,
    pub is_constructor: bool,
    pub code: GMRef<Code>,
}

impl GMElement for Script {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let mut code_id: i32 = reader.read_i32()?;
        let mut is_constructor: bool = false;
        if code_id < -1 {
            code_id &= 0x7FFF_FFFF;
            is_constructor = true;
        }
        let code: GMRef<Code> = GMRef::new(code_id);
        Ok(Self { name, is_constructor, code })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(self.name)?;
        if self.is_constructor {
            if self.code.is_some() {
                builder.write_u32(self.code.index as u32 | 0x8000_0000);
            } else {
                builder.write_i32(-1);
            }
        } else {
            builder.write_resource_id(self.code);
        }
        Ok(())
    }
}
