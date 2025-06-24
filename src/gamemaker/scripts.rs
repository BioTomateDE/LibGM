use crate::gm_deserialize::{DataReader, GMChunkElement, GMElement, GMRef};
use crate::gamemaker::code::GMCode;
use crate::gm_serialize::{instance_muid, DataBuilder};

#[derive(Debug, Clone)]
pub struct GMScripts {
    pub scripts: Vec<GMScript>,
    pub exists: bool,
}
impl GMChunkElement for GMScripts {
    fn empty() -> Self {
        Self { scripts: vec![], exists: false }
    }
}
impl GMElement for GMScripts {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let scripts: Vec<GMScript> = reader.read_scripts_with_occurrences()?;
        Ok(Self { scripts, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        if !self.exists { return Ok(()) }
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
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let mut code_id: i32 = reader.read_i32()?;
        let is_constructor: bool;
        if code_id < -1 {
            code_id = (code_id as u32 & 0x7FFFFFFF) as i32;
            is_constructor = true;
        } else {
            is_constructor = false;
        };

        let code: Option<GMRef<GMCode>> = if code_id == -1 {
            None
        } else {
            let code_id = u32::try_from(code_id).map_err(|e| format!(
                "Could not convert Code ID {code_id} (0x{code_id:08X}) to u32 for Script \"{}\": {e}", reader.display_gm_str(name),
            ))?;
            Some(GMRef::new(code_id))
        };

        Ok(GMScript { name, is_constructor, code })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.resolve_pointer_elem(self)?;
        builder.write_gm_string(&self.name)?;
        if self.is_constructor {
            if let Some(ref gm_code_ref) = self.code {
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

