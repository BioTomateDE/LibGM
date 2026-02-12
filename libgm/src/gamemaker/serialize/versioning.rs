use crate::{
    gamemaker::{elements::GMElement, serialize::builder::DataBuilder, version::GMVersionReq},
    prelude::*,
    util::fmt::typename,
};

impl DataBuilder<'_> {
    pub fn write_if_ver<T, V>(
        &mut self,
        element: &Option<T>,
        field_name: &'static str,
        ver_req: V,
    ) -> Result<()>
    where
        T: GMElement,
        V: Into<GMVersionReq>,
    {
        let ver_req: GMVersionReq = ver_req.into();
        if !self.is_version_at_least(ver_req.clone()) {
            return Ok(()); // Don't serialize if version requirement not met
        }

        let element: &T = element.as_ref().ok_or_else(|| {
            format!(
                "Field {:?} ({}) needs to be set since GameMaker version {} (data version is {})",
                field_name,
                typename::<T>(),
                ver_req,
                self.gm_data.general_info.version,
            )
        })?;

        element.serialize(self)
    }

    pub fn write_if_wad_ver<T>(
        &mut self,
        element: &Option<T>,
        field_name: &'static str,
        ver_req: u8,
    ) -> Result<()>
    where
        T: GMElement,
    {
        if self.wad_version() < ver_req {
            return Ok(()); // Don't serialize if version requirement not met
        }

        let element: &T = element.as_ref().ok_or_else(|| {
            format!(
                "Field {:?} ({}) needs to be set since WAD version {} (data WAD version is {})",
                field_name,
                typename::<T>(),
                ver_req,
                self.gm_data.general_info.wad_version,
            )
        })?;

        element.serialize(self)
    }
}
