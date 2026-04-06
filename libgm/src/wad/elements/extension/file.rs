use crate::prelude::*;
use crate::util::init::num_enum_from;
use crate::wad::deserialize::reader::DataReader;
use crate::wad::elements::GMElement;
use crate::wad::elements::extension::Kind;
use crate::wad::elements::extension::function::Function;
use crate::wad::serialize::builder::DataBuilder;

#[derive(Debug, Clone, PartialEq)]
pub struct File {
    pub filename: String,
    pub cleanup_script: String,
    pub init_script: String,
    pub kind: Kind,
    pub functions: Vec<Function>,
}

impl GMElement for File {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let filename: String = reader.read_gm_string()?;
        let cleanup_script: String = reader.read_gm_string()?;
        let init_script: String = reader.read_gm_string()?;
        let kind: Kind = num_enum_from(reader.read_i32()?)?;
        let functions: Vec<Function> = reader.read_pointer_list()?;
        Ok(Self {
            filename,
            cleanup_script,
            init_script,
            kind,
            functions,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.filename);
        builder.write_gm_string(&self.cleanup_script);
        builder.write_gm_string(&self.init_script);
        builder.write_i32(self.kind.into());
        builder.write_pointer_list(&self.functions)?;
        Ok(())
    }
}
