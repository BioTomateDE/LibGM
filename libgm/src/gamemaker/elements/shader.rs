use macros::named_list_chunk;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{
    gamemaker::{
        deserialize::reader::DataReader, elements::GMElement, serialize::builder::DataBuilder,
    },
    prelude::*,
    util::init::{num_enum_from, vec_with_capacity},
};

#[named_list_chunk("SHDR")]
pub struct GMShaders {
    pub shaders: Vec<GMShader>,
    pub exists: bool,
}

impl GMElement for GMShaders {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        // Figure out where the starts/ends of each shader object are
        let count = reader.read_u32()?;
        let mut locations: Vec<u32> = vec_with_capacity(count + 1)?;
        for _ in 0..count {
            let pointer = reader.read_u32()?;
            if pointer != 0 {
                locations.push(pointer);
            }
        }
        locations.push(reader.chunk.end_pos);

        let mut shaders: Vec<GMShader> = vec_with_capacity(count)?;
        for win in locations.windows(2) {
            let pointer = win[0];
            let entry_end = win[1];
            reader.cur_pos = pointer;
            let name: String = reader.read_gm_string()?;
            let shader_type: GMShaderType = num_enum_from(reader.read_u32()? & 0x7FFF_FFFF)?;

            let glsl_es_vertex: String = reader.read_gm_string()?;
            let glsl_es_fragment: String = reader.read_gm_string()?;
            let glsl_vertex: String = reader.read_gm_string()?;
            let glsl_fragment: String = reader.read_gm_string()?;
            let hlsl9_vertex: String = reader.read_gm_string()?;
            let hlsl9_fragment: String = reader.read_gm_string()?;

            let hlsl11_vertex_ptr = reader.read_u32()?;
            let hlsl11_pixel_ptr = reader.read_u32()?;

            let vertex_shader_attributes: Vec<String> = reader.read_simple_list()?;

            let mut version: i32 = 2;
            let mut pssl_vertex_ptr = 0;
            let mut pssl_vertex_len = 0;
            let mut pssl_pixel_ptr = 0;
            let mut pssl_pixel_len = 0;
            let mut cg_psvita_vertex_ptr = 0;
            let mut cg_psvita_vertex_len = 0;
            let mut cg_psvita_pixel_ptr = 0;
            let mut cg_psvita_pixel_len = 0;
            let mut cg_ps3_vertex_ptr = 0;
            let mut cg_ps3_vertex_len = 0;
            let mut cg_ps3_pixel_ptr = 0;
            let mut cg_ps3_pixel_len = 0;

            if reader.general_info.wad_version > 13 {
                version = reader.read_i32()?;
                pssl_vertex_ptr = reader.read_u32()?;
                pssl_vertex_len = reader.read_u32()?;
                pssl_pixel_ptr = reader.read_u32()?;
                pssl_pixel_len = reader.read_u32()?;
                cg_psvita_vertex_ptr = reader.read_u32()?;
                cg_psvita_vertex_len = reader.read_u32()?;
                cg_psvita_pixel_ptr = reader.read_u32()?;
                cg_psvita_pixel_len = reader.read_u32()?;

                if version >= 2 {
                    cg_ps3_vertex_ptr = reader.read_u32()?;
                    cg_ps3_vertex_len = reader.read_u32()?;
                    cg_ps3_pixel_ptr = reader.read_u32()?;
                    cg_ps3_pixel_len = reader.read_u32()?;
                }
            }

            let hlsl11_vertex_data: Option<ShaderData> =
                read_shader_data(reader, entry_end, 8, hlsl11_vertex_ptr, 0, hlsl11_pixel_ptr)?;
            let hlsl11_pixel_data: Option<ShaderData> =
                read_shader_data(reader, entry_end, 8, hlsl11_pixel_ptr, 0, pssl_vertex_ptr)?;
            let pssl_vertex_data: Option<ShaderData> = read_shader_data(
                reader,
                entry_end,
                8,
                pssl_vertex_ptr,
                pssl_vertex_len,
                pssl_pixel_ptr,
            )?;
            let pssl_pixel_data: Option<ShaderData> = read_shader_data(
                reader,
                entry_end,
                8,
                pssl_pixel_ptr,
                pssl_pixel_len,
                cg_psvita_vertex_ptr,
            )?;
            let cg_psvita_vertex_data: Option<ShaderData> = read_shader_data(
                reader,
                entry_end,
                8,
                cg_psvita_vertex_ptr,
                cg_psvita_vertex_len,
                cg_psvita_pixel_ptr,
            )?;
            let cg_psvita_pixel_data: Option<ShaderData> = read_shader_data(
                reader,
                entry_end,
                8,
                cg_psvita_pixel_ptr,
                cg_psvita_pixel_len,
                cg_ps3_vertex_ptr,
            )?;
            let cg_ps3_vertex_data: Option<ShaderData> = read_shader_data(
                reader,
                entry_end,
                16,
                cg_ps3_vertex_ptr,
                cg_ps3_vertex_len,
                cg_ps3_pixel_ptr,
            )?;
            let cg_ps3_pixel_data: Option<ShaderData> =
                read_shader_data(reader, entry_end, 16, cg_ps3_pixel_ptr, cg_ps3_pixel_len, 0)?;

            shaders.push(GMShader {
                name,
                shader_type,
                glsl_es_vertex,
                glsl_es_fragment,
                glsl_vertex,
                glsl_fragment,
                hlsl9_vertex,
                hlsl9_fragment,
                version,
                hlsl11_vertex_data,
                hlsl11_pixel_data,
                pssl_vertex_data,
                pssl_pixel_data,
                cg_psvita_vertex_data,
                cg_psvita_pixel_data,
                cg_ps3_vertex_data,
                cg_ps3_pixel_data,
                vertex_shader_attributes,
            });
        }

        Ok(Self { shaders, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_pointer_list(&self.shaders)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GMShader {
    pub name: String,
    pub shader_type: GMShaderType,
    pub glsl_es_vertex: String,
    pub glsl_es_fragment: String,
    pub glsl_vertex: String,
    pub glsl_fragment: String,
    pub hlsl9_vertex: String,
    pub hlsl9_fragment: String,
    pub version: i32,
    pub hlsl11_vertex_data: Option<ShaderData>,
    pub hlsl11_pixel_data: Option<ShaderData>,
    pub pssl_vertex_data: Option<ShaderData>,
    pub pssl_pixel_data: Option<ShaderData>,
    pub cg_psvita_vertex_data: Option<ShaderData>,
    pub cg_psvita_pixel_data: Option<ShaderData>,
    pub cg_ps3_vertex_data: Option<ShaderData>,
    pub cg_ps3_pixel_data: Option<ShaderData>,
    pub vertex_shader_attributes: Vec<String>,
}

impl GMElement for GMShader {
    fn deserialize(_: &mut DataReader) -> Result<Self> {
        unimplemented!("GMShader::deserialize is not supported; use GMShaders::deserialize instead")
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.name);
        builder.write_u32(u32::from(self.shader_type) | 0x8000_0000);
        builder.write_gm_string(&self.glsl_es_vertex);
        builder.write_gm_string(&self.glsl_es_fragment);
        builder.write_gm_string(&self.glsl_vertex);
        builder.write_gm_string(&self.glsl_fragment);
        builder.write_gm_string(&self.hlsl9_vertex);
        builder.write_gm_string(&self.hlsl9_fragment);

        builder.write_pointer_opt(&self.hlsl11_vertex_data);
        builder.write_pointer_opt(&self.hlsl11_pixel_data);

        builder.write_simple_list(&self.vertex_shader_attributes)?;

        if builder.wad_version() > 13 {
            builder.write_i32(self.version);
            builder.write_pointer_opt(&self.pssl_vertex_data);
            builder.write_usize(self.pssl_vertex_data.as_ref().map_or(0, |i| i.data.len()))?;
            builder.write_pointer_opt(&self.pssl_pixel_data);
            builder.write_usize(self.pssl_pixel_data.as_ref().map_or(0, |i| i.data.len()))?;
            builder.write_pointer_opt(&self.cg_psvita_vertex_data);
            builder.write_usize(
                self.cg_psvita_vertex_data
                    .as_ref()
                    .map_or(0, |i| i.data.len()),
            )?;
            builder.write_pointer_opt(&self.cg_psvita_pixel_data);
            builder.write_usize(
                self.cg_psvita_pixel_data
                    .as_ref()
                    .map_or(0, |i| i.data.len()),
            )?;
            if self.version >= 2 {
                builder.write_pointer_opt(&self.cg_ps3_vertex_data);
                builder
                    .write_usize(self.cg_ps3_vertex_data.as_ref().map_or(0, |i| i.data.len()))?;
                builder.write_pointer_opt(&self.cg_ps3_pixel_data);
                builder.write_usize(self.cg_ps3_pixel_data.as_ref().map_or(0, |i| i.data.len()))?;
            }
        }

        write_shader_data(builder, 8, &self.hlsl11_vertex_data)?;
        write_shader_data(builder, 8, &self.hlsl11_pixel_data)?;
        write_shader_data(builder, 8, &self.pssl_vertex_data)?;
        write_shader_data(builder, 8, &self.pssl_pixel_data)?;
        write_shader_data(builder, 8, &self.cg_psvita_vertex_data)?;
        write_shader_data(builder, 8, &self.cg_psvita_pixel_data)?;
        write_shader_data(builder, 16, &self.cg_ps3_vertex_data)?;
        write_shader_data(builder, 16, &self.cg_ps3_pixel_data)?;

        Ok(())
    }
}

/// Possible shader types a shader can have.
/// All console shaders (and HLSL11?) are compiled using confidential SDK tools when
/// `GMAssetCompiler` builds the game (for PSVita it's `psp2cgc` shader compiler).
#[derive(Debug, Clone, Copy, TryFromPrimitive, IntoPrimitive, PartialEq, Eq)]
#[repr(u32)]
pub enum GMShaderType {
    GlslEs = 1,
    GLSL = 2,
    HLSL9 = 3,
    HLSL11 = 4,
    /// PSSL is a shading language used only in PS4, based on HLSL11.
    PSSL = 5,
    /// Cg stands for "C for graphics" made by NVIDIA and used in PSVita and PS3 (they have their own variants of Cg), based on HLSL9.
    CgPsvita = 6,
    /// Cg stands for "C for graphics" made by NVIDIA and used in PSVita and PS3 (they have their own variants of Cg), based on HLSL9.
    CgPs3 = 7,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderData {
    pub data: Vec<u8>,
}

fn read_shader_data(
    reader: &mut DataReader,
    entry_end: u32,
    pad: u32,
    this_pointer: u32,
    expected_length: u32,
    next_ptr: u32,
) -> Result<Option<ShaderData>> {
    const ERR_MSG_PREFIX: &str = "Failed to compute length of shader data: instructed to read";
    const ERR_MSG_SUFFIX: &str =
        "Shader data was the last in the shader, but given length was incorrectly padded.";

    if this_pointer == 0 {
        return Ok(None);
    }

    reader.align(pad)?;
    let next = if next_ptr == 0 { entry_end } else { next_ptr };
    let actual_length = next - reader.cur_pos;
    let is_last: bool = next_ptr == 0;

    if expected_length == 0 {
        let data: Vec<u8> = reader.read_bytes_dyn(actual_length)?.to_vec();
        return Ok(Some(ShaderData { data }));
    }

    if expected_length > actual_length {
        bail!("{ERR_MSG_PREFIX} less data than expected.");
    }

    if expected_length < actual_length {
        if is_last && (reader.cur_pos + actual_length).is_multiple_of(16) {
            // Normal for the last element due to chunk padding, just trust the system
        } else if !is_last && (reader.cur_pos + actual_length).is_multiple_of(8) {
            // Normal for 8-byte alignment to occur on all elements prior to the last one
        } else if is_last {
            bail!("{ERR_MSG_PREFIX} more data than expected. {ERR_MSG_SUFFIX}");
        } else {
            bail!("{ERR_MSG_PREFIX} more data than expected.");
        }
    }

    let data: Vec<u8> = reader.read_bytes_dyn(expected_length)?.to_vec();
    Ok(Some(ShaderData { data }))
}

fn write_shader_data(
    builder: &mut DataBuilder,
    pad: u32,
    shader_data_opt: &Option<ShaderData>,
) -> Result<()> {
    if let Some(shader_data) = shader_data_opt {
        builder.align(pad);
        builder.resolve_pointer(shader_data)?;
        builder.write_bytes(&shader_data.data);
    }
    Ok(())
}
