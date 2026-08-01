// SPDX-License-Identifier: GPL-3.0-only
mod file;
pub mod function;
pub mod option;

pub use self::file::File;
pub use self::function::Function;
pub use self::option::ExtOption;
use crate::gm_enum::gm_enum;
use crate::prelude::*;
use crate::wad::Blob;
use crate::wad::chunk::gm_asset_list_chunk;
use crate::wad::elem::GMElement;
use crate::wad::load::reader::DataReader;
use crate::wad::save::writer::DataWriter;
use crate::wad::version::GMVersion;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Extensions {
    pub elems: Vec<Option<Extension>>,

    // Strange data for each extension, some kind of unique identifier based on
    // the product ID for each of them.
    //
    // This should should have same number of elements as `elems`.
    pub product_id_data: Vec<Blob<[u8; 16]>>,
}

// not sure if nullable
gm_asset_list_chunk!(EXTN, Extensions, Extension);

impl GMElement for Extensions {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let elems: Vec<Option<Extension>> = reader.read_pointer_list_opt()?;
        let mut product_id_data = Vec::new();

        if has_product_id_data(reader.version) {
            product_id_data = Vec::with_capacity(elems.len());
            for _ in 0..elems.len() {
                let bytes: [u8; 16] = *reader.read_bytes_const()?;
                product_id_data.push(Blob(bytes));
            }
        }

        Ok(Self { elems, product_id_data })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_pointer_list_opt(&self.elems)?;

        if has_product_id_data(writer.version()) {
            let extension_count = self.elems.len();
            let product_data_count = self.product_id_data.len();
            if extension_count != product_data_count {
                bail!(
                    "Expected {extension_count} product ID data entries, got {product_data_count}"
                );
            }
            for data in &self.product_id_data {
                writer.write_bytes(&data.0);
            }
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
        let version: GMRef<String> =
            if reader.version == GMVersion::Lts2022_0_3 || reader.version >= GMVersion::GM2023_4 {
                reader.read_gm_string()?
            } else {
                GMRef::none()
            };
        let class_name: GMRef<String> = reader.read_gm_string()?;
        let files: Vec<File>;
        let options: Vec<ExtOption>;

        if reader.version >= GMVersion::GM2022_6 {
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

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_gm_string(self.folder_name)?;
        writer.write_gm_string(self.name)?;
        if writer.version() == GMVersion::Lts2022_0_3 || writer.version() >= GMVersion::GM2023_4 {
            writer.write_gm_string(self.version)?;
        }
        writer.write_gm_string(self.class_name)?;
        if writer.version() >= GMVersion::GM2022_6 {
            writer.write_pointer(&self.files);
            writer.write_pointer(&self.options);

            writer.resolve_pointer(&self.files)?;
            writer.write_pointer_list(&self.files)?;

            writer.resolve_pointer(&self.options)?;
            writer.write_pointer_list(&self.options)?;
        } else {
            writer.write_pointer_list(&self.files)?;
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

fn has_product_id_data(ver: GMVersion) -> bool {
    ver >= GMVersion::Wad16 || ver == GMVersion::Wad14Beta
}
