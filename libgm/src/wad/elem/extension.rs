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
pub struct Extensions {
    pub elems: Vec<Extension>,
    pub exists: bool,
}

// not sure if nullable
gm_named_list_chunk!(EXTN, Extensions, Extension, direct);

impl GMElement for Extensions {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let mut elems: Vec<Extension> = reader.read_pointer_list()?;

        if product_id_data_eligible(reader.general_info.version) {
            for elem in &mut elems {
                let bytes: [u8; 16] = reader.read_bytes_const()?.to_owned();
                elem.product_id_data = Some(bytes);
            }
        }

        Ok(Self { elems, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_pointer_list(&self.elems)?;

        if !product_id_data_eligible(builder.version()) {
            return Ok(());
        }

        for elem in &self.elems {
            let Some(data) = &elem.product_id_data else {
                bail!(
                    "Extension's Product ID Data is not set in {}",
                    builder.version()
                );
            };
            builder.write_bytes(data);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Extension {
    pub folder_name: GMRef<String>,

    pub name: GMRef<String>,

    /// Present in 2023.4+
    pub version: GMRef<String>,

    pub class_name: GMRef<String>,

    pub files: Vec<File>,

    /// Present in 2022.6+
    pub options: Vec<ExtOption>,

    /// Strange data for each extension, some kind of unique
    /// identifier based on the product ID for each of them.
    ///
    /// Present in GMS2+ (and some scuffed GMS1 versions)
    pub product_id_data: Option<[u8; 16]>,
}

impl GMElement for Extension {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let folder_name: GMRef<String> = reader.read_gm_string()?;
        let name: GMRef<String> = reader.read_gm_string()?;
        let version: GMRef<String> = if reader.general_info.version >= (2023, 4) {
            reader.read_gm_string()?
        } else {
            GMRef::none()
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
            product_id_data: None,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(self.folder_name)?;
        builder.write_gm_string(self.name)?;
        if builder.version() >= (2023, 4) {
            builder.write_gm_string(self.version)?;
        }
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
    Unknown2 = 12,
});

#[must_use]
const fn product_id_data_eligible(ver: GMVersion) -> bool {
    // NOTE: I do not know if 1773 is the earliest version which contains product IDs.
    ver.major >= 2 || (ver.major == 1 && ver.build >= 1773) || (ver.major == 1 && ver.build == 1539)
}
