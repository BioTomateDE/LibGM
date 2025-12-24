mod file;
pub mod function;
pub mod option;

pub use file::File;
pub use function::Function;
use macros::num_enum;

use crate::{
    gamemaker::{
        chunk::ChunkName,
        deserialize::reader::DataReader,
        elements::{GMChunk, GMElement},
        gm_version::GMVersion,
        serialize::{builder::DataBuilder, traits::GMSerializeIfVersion},
    },
    prelude::*,
};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GMExtensions {
    pub extensions: Vec<GMExtension>,
    /// Set in GMS2+ (and some scuffed GMS1 versions)
    pub product_id_data: Vec<[u8; 16]>,
    pub exists: bool,
}

impl GMChunk for GMExtensions {
    const NAME: ChunkName = ChunkName::new("EXTN");
    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMElement for GMExtensions {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let extensions: Vec<GMExtension> = reader.read_pointer_list()?;

        // Strange data for each extension, some kind of unique
        // identifier based on the product ID for each of them.
        let mut product_id_data = Vec::new();
        if product_id_data_eligible(&reader.general_info.version) {
            product_id_data.reserve(extensions.len());
            for _ in 0..extensions.len() {
                let bytes: [u8; 16] = reader.read_bytes_const()?.to_owned();
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

        let ext_count = self.extensions.len();
        let prod_count = self.product_id_data.len();

        if prod_count > ext_count {
            bail!(
                "There are more Product ID data than extensions: {} > {}",
                prod_count,
                ext_count,
            )
        }
        if prod_count < ext_count {
            log::warn!(
                "The last {ext_count} extension don't have any \
                Product ID data; null bytes will be written instead"
            );
        }

        for data in &self.product_id_data {
            builder.write_bytes(data);
        }

        // Potentially write null bytes for extensions
        // that don't have any product id data (yet).
        for _ in ext_count..prod_count {
            builder.write_bytes(&[0u8; 16]);
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
    pub files: Vec<File>,
    /// Present in 2022.6+
    pub options: Vec<option::Option>,
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
        let files: Vec<File>;
        let options: Vec<option::Option>;

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
        self.version
            .serialize_if_gm_ver(builder, "Version", (2023, 4))?;
        builder.write_gm_string(&self.class_name);
        if builder.is_gm_version_at_least((2022, 6)) {
            builder.write_pointer(&self.files);
            builder.write_pointer(&self.options);

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

#[num_enum(i32)]
pub enum Kind {
    Dll = 1,
    GML = 2,
    ActionLib = 3,
    Generic = 4,
    JavaScript = 5,
}

#[must_use]
const fn product_id_data_eligible(ver: &GMVersion) -> bool {
    // NOTE: I do not know if 1773 is the earliest version which contains product IDs.
    ver.major >= 2 || (ver.major == 1 && ver.build >= 1773) || (ver.major == 1 && ver.build == 1539)
}
