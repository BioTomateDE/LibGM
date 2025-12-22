use crate::{
    gamemaker::{elements::GMElement, gm_version::GMVersionReq, serialize::builder::DataBuilder},
    prelude::*,
    util::fmt::typename,
};

pub trait GMSerializeIfVersion {
    fn serialize_if_gm_ver<V: Into<GMVersionReq>>(
        &self,
        builder: &mut DataBuilder,
        field_name: &'static str,
        ver_req: V,
    ) -> Result<()>;

    fn serialize_if_bytecode_ver(
        &self,
        builder: &mut DataBuilder,
        field_name: &'static str,
        ver_req: u8,
    ) -> Result<()>;
}

/// TODO: these trait are kind of ass, move them to [`DataBuilder`].
///       maybe like `builder.write_if_ver(element, (2023, 6), "")?;`
impl<T: GMElement> GMSerializeIfVersion for Option<T> {
    fn serialize_if_gm_ver<V: Into<GMVersionReq>>(
        &self,
        builder: &mut DataBuilder,
        field_name: &'static str,
        ver_req: V,
    ) -> Result<()> {
        let ver_req: GMVersionReq = ver_req.into();
        if !builder.is_gm_version_at_least(ver_req.clone()) {
            return Ok(()); // Don't serialize if version requirement not met
        }
        let element: &T = self.as_ref().ok_or_else(|| {
            format!(
                "Field '{}' of {} is not set in data with GameMaker version {} \
                but needs to be set since GameMaker version {}",
                field_name,
                typename::<T>(),
                builder.gm_data.general_info.version,
                ver_req,
            )
        })?;
        element.serialize(builder)
    }

    fn serialize_if_bytecode_ver(
        &self,
        builder: &mut DataBuilder,
        field_name: &'static str,
        ver_req: u8,
    ) -> Result<()> {
        if builder.bytecode_version() < ver_req {
            return Ok(()); // Don't serialize if version requirement not met
        }
        let element: &T = self.as_ref().ok_or_else(|| {
            format!(
                "Field '{}' of {} is not set in data with Bytecode version {}
                but needs to be set since Bytecode version {}",
                field_name,
                typename::<T>(),
                builder.gm_data.general_info.bytecode_version,
                ver_req,
            )
        })?;
        element.serialize(builder)
    }
}
