use num_enum::{IntoPrimitive, TryFromPrimitive};
use crate::gm_deserialize::{DataReader, GMChunkElement, GMElement, GMRef};
use crate::gm_serialize::DataBuilder;

#[derive(Debug, Clone)]
pub struct GMExtensions {
    pub extensions: Vec<GMExtension>,
    pub exists: bool,
}
impl GMChunkElement for GMExtensions {
    fn empty() -> Self {
        Self { extensions: vec![], exists: false }
    }
    fn exists(&self) -> bool {
        self.exists
    }
}
impl GMElement for GMExtensions {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let extensions: Vec<GMExtension> = reader.read_pointer_list()?;
        Ok(GMExtensions { extensions, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        if !self.exists { return Ok(()) }
        builder.write_pointer_list(&self.extensions)?;
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMExtension {
    pub name: GMRef<String>,
    pub value: GMRef<String>,
    pub kind: GMExtensionOptionKind,
}
impl GMElement for GMExtension {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let value: GMRef<String> = reader.read_gm_string()?;
        let kind: u32 = reader.read_u32()?;
        let kind: GMExtensionOptionKind = kind.try_into().map_err(|_| format!("Invalid Extension Option Kind {kind} (0x{kind:08X})"))?;
        Ok(GMExtension { name, value, kind })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_gm_string(&self.name)?;
        builder.write_gm_string(&self.value)?;
        builder.write_u32(self.kind.into());
        Ok(())
    }
}


#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum GMExtensionOptionKind {
    Boolean = 0,
    Number = 1,
    String = 2,
}

