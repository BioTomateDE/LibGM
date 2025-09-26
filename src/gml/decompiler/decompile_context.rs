use crate::gamemaker::data::GMData;

#[derive(Debug, Clone)]
pub struct DecompileContext<'a> {
    pub gm_data: &'a GMData,
}

