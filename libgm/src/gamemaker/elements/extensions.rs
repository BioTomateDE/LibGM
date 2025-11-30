use std::ops::{Deref, DerefMut};

use macros::num_enum;

use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{GMChunkElement, GMElement},
        gm_version::GMVersion,
        serialize::builder::DataBuilder,
    },
    prelude::*,
    util::init::num_enum_from,
};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GMExtensions {
    pub extensions: Vec<GMExtension>,
    /// only set in gms2 (and some scuffed gms1 versions)
    pub product_id_data: Vec<[u8; 16]>,
    pub exists: bool,
}

impl Deref for GMExtensions {
    type Target = Vec<GMExtension>;
    fn deref(&self) -> &Self::Target {
        &self.extensions
    }
}

impl DerefMut for GMExtensions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.extensions
    }
}

impl GMChunkElement for GMExtensions {
    const NAME: &'static str = "EXTN";
    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMElement for GMExtensions {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let extensions: Vec<GMExtension> = reader.read_pointer_list()?;

        // Strange data for each extension, some kind of unique identifier based on the product ID for each of them
        let mut product_id_data = Vec::with_capacity(extensions.len());
        if product_id_data_eligible(&reader.general_info.version) {
            for _ in 0..extensions.len() {
                let bytes: [u8; 16] = reader.read_bytes_const::<16>()?.to_owned();
                product_id_data.push(bytes);
            }
        }

        Ok(Self {
            extensions,
            product_id_data,
            exists: true,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_pointer_list(&self.extensions)?;

        if !product_id_data_eligible(&builder.gm_data.general_info.version) {
            return Ok(());
        }

        let extension_count = self.extensions.len();
        let product_id_count = self.product_id_data.len();

        if product_id_count > extension_count {
            bail!(
                "More Product ID data than extensions: {} > {}",
                product_id_count,
                extension_count,
            )
        }
        if product_id_count < extension_count {
            log::warn!(
                "The last {extension_count} extensions don't have any Product ID data; null bytes will be written instead"
            );
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
    pub folder_name: String,
    pub name: String,
    /// Present in 2023.4+
    pub version: Option<String>,
    pub class_name: String,
    pub files: Vec<GMExtensionFile>,
    /// Present in 2022.6+
    pub options: Vec<GMExtensionOption>,
}

impl GMElement for GMExtension {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let folder_name: String = reader.read_gm_string()?;
        let name: String = reader.read_gm_string()?;
        let version: Option<String> = if reader.general_info.is_version_at_least((2023, 4)) {
            Some(reader.read_gm_string()?)
        } else {
            None
        };
        let class_name: String = reader.read_gm_string()?;
        let files: Vec<GMExtensionFile>;
        let options: Vec<GMExtensionOption>;

        if reader.general_info.is_version_at_least((2022, 6)) {
            let files_ptr = reader.read_u32()?;
            let options_ptr = reader.read_u32()?;

            reader.assert_pos(files_ptr, "Files")?;
            files = reader.read_pointer_list()?;

            reader.assert_pos(options_ptr, "Options")?;
            options = reader.read_pointer_list()?;
        } else {
            files = reader.read_pointer_list()?;
            options = Vec::new();
        }

        Ok(Self {
            folder_name,
            name,
            version,
            class_name,
            files,
            options,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.folder_name);
        builder.write_gm_string(&self.name);
        if builder.is_gm_version_at_least((2023, 4)) {
            let version = self
                .version
                .as_ref()
                .ok_or("Extension Version not set in 2023.4+")?;
            builder.write_gm_string(version);
        }
        builder.write_gm_string(&self.class_name);
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
    pub filename: String,
    pub cleanup_script: String,
    pub init_script: String,
    pub kind: GMExtensionKind,
    pub functions: Vec<GMExtensionFunction>,
}

impl GMElement for GMExtensionFile {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let filename: String = reader.read_gm_string()?;
        let cleanup_script: String = reader.read_gm_string()?;
        let init_script: String = reader.read_gm_string()?;
        let kind: GMExtensionKind = num_enum_from(reader.read_i32()?)?;
        let functions: Vec<GMExtensionFunction> = reader.read_pointer_list()?;
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

#[derive(Debug, Clone, PartialEq)]
pub struct GMExtensionFunction {
    pub name: String,
    pub id: u32,
    pub kind: GMExtensionKind,
    pub return_type: GMExtensionReturnType,
    pub ext_name: String,
    pub arguments: Vec<GMExtensionFunctionArg>,
}

impl GMElement for GMExtensionFunction {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: String = reader.read_gm_string()?;
        let id = reader.read_u32()?;
        let kind: GMExtensionKind = num_enum_from(reader.read_i32()?)?; // Assumption; UTMT uses u32
        let return_type: GMExtensionReturnType = num_enum_from(reader.read_i32()?)?;
        let ext_name: String = reader.read_gm_string()?;
        let arguments: Vec<GMExtensionFunctionArg> = reader.read_simple_list()?;
        Ok(Self {
            name,
            id,
            kind,
            return_type,
            ext_name,
            arguments,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.name);
        builder.write_u32(self.id);
        builder.write_i32(self.kind.into());
        builder.write_i32(self.return_type.into());
        builder.write_gm_string(&self.ext_name);
        builder.write_simple_list(&self.arguments)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GMExtensionFunctionArg {
    pub return_type: GMExtensionReturnType,
}

impl GMElement for GMExtensionFunctionArg {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let return_type: GMExtensionReturnType = num_enum_from(reader.read_i32()?)?;
        Ok(Self { return_type })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i32(self.return_type.into());
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GMExtensionOption {
    pub name: String,
    pub value: String,
    pub kind: GMExtensionOptionKind,
}

impl GMElement for GMExtensionOption {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: String = reader.read_gm_string()?;
        let value: String = reader.read_gm_string()?;
        let kind: GMExtensionOptionKind = num_enum_from(reader.read_i32()?)?;
        Ok(Self { name, value, kind })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.name);
        builder.write_gm_string(&self.value);
        builder.write_i32(self.kind.into());
        Ok(())
    }
}

#[num_enum(i32)]
pub enum GMExtensionKind {
    Dll = 1,
    GML = 2,
    ActionLib = 3,
    Generic = 4,
    JavaScript = 5,
}

#[num_enum(i32)]
pub enum GMExtensionReturnType {
    String = 1,
    Double = 2,
}

#[num_enum(i32)]
pub enum GMExtensionOptionKind {
    Boolean = 0,
    Number = 1,
    String = 2,
}

#[must_use]
const fn product_id_data_eligible(ver: &GMVersion) -> bool {
    // NOTE: I do not know if 1773 is the earliest version which contains product IDs.
    ver.major >= 2 || (ver.major == 1 && ver.build >= 1773) || (ver.major == 1 && ver.build == 1539)
}
