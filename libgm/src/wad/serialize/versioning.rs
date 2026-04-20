use crate::prelude::*;
use crate::util::fmt::typename;
use crate::wad::elements::GMElement;
use crate::wad::serialize::builder::DataBuilder;
use crate::wad::version::ToGMVersion;

// The element fields store `Option<T>`, so passing `Option<&T>` instead
// of `&Option<T>` would require using `.as_ref()` in every call.
#[allow(clippy::ref_option)]
impl DataBuilder<'_> {
    #[inline]
    pub fn write_if_ver<T: GMElement>(
        &mut self,
        element: &Option<T>,
        field_name: &'static str,
        ver_req: impl ToGMVersion,
    ) -> Result<()> {
        let ver_req = ver_req.to_gm_version();
        if self.version() < ver_req {
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

    #[inline]
    pub fn write_if_wad_ver<T: GMElement>(
        &mut self,
        element: &Option<T>,
        field_name: &'static str,
        ver_req: u8,
    ) -> Result<()> {
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
