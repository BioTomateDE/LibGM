use num_enum::{IntoPrimitive, TryFromPrimitive};
use crate::gamemaker::deserialize::{DataReader, GMRef};
use crate::gamemaker::element::{GMChunkElement, GMElement};
use crate::gamemaker::serialize::DataBuilder;
use crate::utility::num_enum_from;

#[derive(Debug, Clone)]
pub struct GMExtensions {
    pub extensions: Vec<GMExtension>,
    /// only set in gms2 (and some scuffed gms1 versions)
    pub product_id_data: Vec<[u8; 16]>,
    pub exists: bool,
}
impl GMChunkElement for GMExtensions {
    fn empty() -> Self {
        Self { extensions: vec![], product_id_data: vec![], exists: false }
    }
    fn exists(&self) -> bool {
        self.exists
    }
}
impl GMElement for GMExtensions {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let extensions: Vec<GMExtension> = reader.read_pointer_list()?;
        
        // Strange data for each extension, some kind of unique identifier based on
        // the product ID for each of them
        // NOTE: I do not know if 1773 is the earliest version which contains product IDs.
        let mut product_id_data = Vec::with_capacity(extensions.len());
        let ver = &reader.general_info.version;
        if ver.major >= 2 || (ver.major == 1 && ver.build >= 1773) || (ver.major == 1 && ver.build == 1539) {
            log::debug!("Scuffed product ID data for extensions detected");
            for _ in 0..extensions.len() {
                let bytes: [u8; 16] = reader.read_bytes_const::<16>()?.to_owned();
                product_id_data.push(bytes); 
            }
        }
        
        Ok(GMExtensions { extensions, product_id_data, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        if !self.exists { return Ok(()) }
        builder.write_pointer_list(&self.extensions)?;
        for data in &self.product_id_data {
            builder.write_bytes(data);
        }
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
        let kind: GMExtensionOptionKind = num_enum_from(reader.read_u32()?)?;
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

