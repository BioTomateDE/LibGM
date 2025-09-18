use num_enum::{IntoPrimitive, TryFromPrimitive};
use crate::gamemaker::deserialize::{DataReader, GMRef};
use crate::gamemaker::element::{GMChunkElement, GMElement};
use crate::gamemaker::gm_version::GMVersion;
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
    fn stub() -> Self {
        Self { extensions: vec![], product_id_data: vec![], exists: false }
    }
    fn exists(&self) -> bool {
        self.exists
    }
}
impl GMElement for GMExtensions {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let extensions: Vec<GMExtension> = reader.read_pointer_list()?;
        
        // Strange data for each extension, some kind of unique identifier based on the product ID for each of them
        let mut product_id_data = Vec::with_capacity(extensions.len());
        if product_id_data_eligible(&reader.general_info.version) {
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

        if !product_id_data_eligible(&builder.gm_data.general_info.version) {
            return Ok(())
        }

        let extension_count = self.extensions.len();
        let product_id_count = self.product_id_data.len();

        if product_id_count > extension_count {
            return Err(format!(
                "More Product ID data than extensions: {} > {}",
                product_id_count, extension_count,
            ))
        }
        if product_id_count < extension_count {
            log::warn!("The last {extension_count} extensions don't have any Product ID data; null bytes will be written instead");
        }

        let mut product_id_data = self.product_id_data.clone();
        product_id_data.resize(extension_count, [0; 16]);
        for data in product_id_data {
            builder.write_bytes(&data);
        }
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMExtension {
    pub folder_name: GMRef<String>,
    pub name: GMRef<String>,
    /// Present in 2023.4+
    pub version: Option<GMRef<String>>,
    pub class_name: GMRef<String>,
    pub files: Vec<GMExtensionFile>,
    /// Present in 2022.6+
    pub options: Vec<GMExtensionOption>,
}
impl GMElement for GMExtension {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let folder_name: GMRef<String> = reader.read_gm_string()?;
        let name: GMRef<String> = reader.read_gm_string()?;
        let version: Option<GMRef<String>> = if reader.general_info.is_version_at_least((2023, 4)) {
            Some(reader.read_gm_string()?)
        } else { None };
        let class_name: GMRef<String> = reader.read_gm_string()?;
        let files: Vec<GMExtensionFile>;
        let mut options: Vec<GMExtensionOption> = Vec::new();

        if reader.general_info.is_version_at_least((2022, 6)) {
            let files_ptr: usize = reader.read_usize()?;
            let options_ptr: usize = reader.read_usize()?;

            reader.assert_pos(files_ptr, "Files")?;
            files = reader.read_pointer_list()?;

            reader.assert_pos(options_ptr, "Options")?;
            options = reader.read_pointer_list()?;
        } else {
            files = reader.read_pointer_list()?;
        }

        Ok(Self { folder_name, name, version, class_name, files, options })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_gm_string(&self.folder_name)?;
        builder.write_gm_string(&self.name)?;
        if builder.is_gm_version_at_least((2023, 4)) {
            let version: GMRef<String> = self.version.ok_or("Extension Version not set in 2023.4+")?;
            builder.write_gm_string(&version)?;
        }
        builder.write_gm_string(&self.class_name)?;
        if builder.is_gm_version_at_least((2022, 6)) {
            builder.write_pointer(&self.files)?;
            builder.write_pointer(&self.options)?;

            builder.resolve_pointer(&self.files)?;
            builder.write_pointer_list(&self.files)?;

            builder.resolve_pointer(&self.options)?;
            builder.write_pointer_list(&self.options)?;
        } else {
            builder.write_pointer_list(&self.files)?;
        }
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMExtensionFile {
    pub filename: GMRef<String>,
    pub cleanup_script: GMRef<String>,
    pub init_script: GMRef<String>,
    pub kind: GMExtensionKind,
    pub functions: Vec<GMExtensionFunction>,
}
impl GMElement for GMExtensionFile {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let filename: GMRef<String> = reader.read_gm_string()?;
        let cleanup_script: GMRef<String> = reader.read_gm_string()?;
        let init_script: GMRef<String> = reader.read_gm_string()?;
        let kind: GMExtensionKind = num_enum_from(reader.read_u32()?)?;
        let functions: Vec<GMExtensionFunction> = reader.read_pointer_list()?;
        Ok(Self { filename, cleanup_script, init_script, kind, functions })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_gm_string(&self.filename)?;
        builder.write_gm_string(&self.cleanup_script)?;
        builder.write_gm_string(&self.init_script)?;
        builder.write_u32(self.kind.into());
        builder.write_pointer_list(&self.functions)?;
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMExtensionFunction {
    pub name: GMRef<String>,
    pub id: u32,
    pub kind: GMExtensionKind,
    pub return_type: GMExtensionReturnType,
    pub ext_name: GMRef<String>,
    pub arguments: Vec<GMExtensionFunctionArg>,
}
impl GMElement for GMExtensionFunction {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let id: u32 = reader.read_u32()?;
        let kind: GMExtensionKind = num_enum_from(reader.read_u32()?)?;    // assumption; UTMT uses u32
        let return_type: GMExtensionReturnType = num_enum_from(reader.read_u32()?)?;
        let ext_name: GMRef<String> = reader.read_gm_string()?;
        let arguments: Vec<GMExtensionFunctionArg> = reader.read_simple_list()?;
        Ok(Self { name, id, kind, return_type, ext_name, arguments })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_gm_string(&self.name)?;
        builder.write_u32(self.id);
        builder.write_u32(self.kind.into());
        builder.write_u32(self.return_type.into());
        builder.write_gm_string(&self.ext_name)?;
        builder.write_simple_list(&self.arguments)?;
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMExtensionFunctionArg {
    pub return_type: GMExtensionReturnType,
}
impl GMElement for GMExtensionFunctionArg {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let return_type: GMExtensionReturnType = num_enum_from(reader.read_u32()?)?;
        Ok(Self { return_type })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_u32(self.return_type.into());
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMExtensionOption {
    pub name: GMRef<String>,
    pub value: GMRef<String>,
    pub kind: GMExtensionOptionKind,
}
impl GMElement for GMExtensionOption {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let value: GMRef<String> = reader.read_gm_string()?;
        let kind: GMExtensionOptionKind = num_enum_from(reader.read_u32()?)?;
        Ok(GMExtensionOption { name, value, kind })
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
pub enum GMExtensionKind {
    Dll = 1,
    GML = 2,
    ActionLib = 3,
    Generic = 4,
    JavaScript = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum GMExtensionReturnType {
    String = 1,
    Double = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum GMExtensionOptionKind {
    Boolean = 0,
    Number = 1,
    String = 2,
}


fn product_id_data_eligible(ver: &GMVersion) -> bool {
    // NOTE: I do not know if 1773 is the earliest version which contains product IDs.
    ver.major >= 2 || (ver.major == 1 && ver.build >= 1773) || (ver.major == 1 && ver.build == 1539)
}

