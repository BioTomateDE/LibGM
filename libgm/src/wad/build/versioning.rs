// SPDX-License-Identifier: GPL-3.0-only
use crate::prelude::*;
use crate::util::fmt::typename;
use crate::wad::GMVersion;
use crate::wad::build::builder::DataBuilder;
use crate::wad::elem::GMElement;

// The element fields store `Option<T>`, so passing `Option<&T>` instead
// of `&Option<T>` would require using `.as_ref()` in every call.
#[allow(clippy::ref_option)]
impl DataBuilder<'_> {
    #[inline]
    pub fn write_if_ver<T: GMElement>(
        &mut self,
        element: &Option<T>,
        field_name: &'static str,
        ver_req: GMVersion,
    ) -> Result<()> {
        if self.version() < ver_req {
            return Ok(()); // Don't serialize if version requirement not met
        }

        let element: &T = element.as_ref().ok_or_else(|| {
            format!(
                "Field {:?} ({}) needs to be set since GameMaker version {} (data version is {})",
                field_name,
                typename::<T>(),
                ver_req,
                self.version(),
            )
        })?;

        element.serialize(self)
    }
}
