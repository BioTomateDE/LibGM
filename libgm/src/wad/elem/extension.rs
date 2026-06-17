// SPDX-License-Identifier: GPL-3.0-only
mod file;
pub mod function;
pub mod option;

pub use self::file::File;
pub use self::function::Function;
pub use self::option::ExtOption;
use crate::gm_enum::gm_enum;
use crate::prelude::*;
use crate::wad::build::builder::DataBuilder;
use crate::wad::chunk::gm_named_list_chunk;
use crate::wad::elem::GMElement;
use crate::wad::parse::reader::DataReader;
use crate::wad::version::GMVersion;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GMExtensions {
    pub elems: Vec<GMExtension>,
    /// Set in GMS2+ (and some scuffed GMS1 versions)
    // TODO: merge into GMExtension
    pub product_id_data: Vec<[u8; 16]>,
    pub exists: bool,
}

// not sure if nullable
gm_named_list_chunk!(EXTN, GMExtensions, GMExtension, direct);

impl GMElement for GMExtensions {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let elems: Vec<GMExtension> = reader.read_pointer_list()?;

        // Strange data for each extension, some kind of unique
        // identifier based on the product ID for each of them.
        let mut product_id_data = Vec::new();
        if product_id_data_eligible(&reader.general_info.version) {
            product_id_data.reserve(elems.len());
            for _ in 0..elems.len() {
                let bytes: [u8; 16] = reader.read_bytes_const()?.to_owned();
                product_id_data.push(bytes);
            }
        }

        Ok(Self { elems, product_id_data, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_pointer_list(&self.elems)?;

        if !product_id_data_eligible(&builder.gm_data.general_info.version) {
            return Ok(());
        }

        let ext_count = self.elems.len();
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
                "The last {ext_count} extension don't have any Product ID data; null bytes will \
                 be written instead"
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
    pub folder_name: GMRef<String>,
    pub name: GMRef<String>,
    /// Present in 2023.4+
    pub version: Option<GMRef<String>>,
    pub class_name: GMRef<String>,
    pub files: Vec<File>,
    /// Present in 2022.6+
    pub options: Vec<ExtOption>,
}

impl GMElement for GMExtension {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let folder_name: GMRef<String> = reader.read_gm_string()?;
        let name: GMRef<String> = reader.read_gm_string()?;
        let version: Option<GMRef<String>> = if reader.general_info.version >= (2023, 4) {
            Some(reader.read_gm_string()?)
        } else {
            None
        };
        let class_name: GMRef<String> = reader.read_gm_string()?;
        let files: Vec<File>;
        let options: Vec<ExtOption>;

        if reader.general_info.version >= (2022, 6) {
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
        builder.write_gm_string(self.folder_name)?;
        builder.write_gm_string(self.name)?;
        builder.write_if_ver(&self.version, "Version", (2023, 4))?;
        builder.write_gm_string(self.class_name)?;
        if builder.version() >= (2022, 6) {
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

gm_enum!( Kind {
    Dll = 1,
    Gml = 2,
    ActionLib = 3,
    Generic = 4,
    JavaScript = 5,

    /// Seems to be modern.
    /// UTMT doesn't specify a name for this.
    ///
    /// NOTE: This will probably be renamed to something better soon.
    Unknown1 = 11,
});

#[must_use]
const fn product_id_data_eligible(ver: &GMVersion) -> bool {
    // NOTE: I do not know if 1773 is the earliest version which contains product
    // IDs.
    ver.major >= 2 || (ver.major == 1 && ver.build >= 1773) || (ver.major == 1 && ver.build == 1539)
}
