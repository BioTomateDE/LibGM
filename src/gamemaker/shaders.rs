use num_enum::{IntoPrimitive, TryFromPrimitive};
use crate::gamemaker::code::GMCode;
use crate::gm_deserialize::{DataReader, GMChunkElement, GMElement, GMRef};
use crate::gm_serialize::DataBuilder;
use crate::utility::{num_enum_from, vec_with_capacity};

#[derive(Debug, Clone)]
pub struct GMShaders {
    pub shaders: Vec<GMRef<GMCode>>,
    pub exists: bool,
}
impl GMChunkElement for GMShaders {
    fn empty() -> Self {
        Self { shaders: vec![], exists: false }
    }
    fn exists(&self) -> bool {
        self.exists
    }
}
impl GMElement for GMShaders {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        // Figure out where the starts/ends of each shader object are
        let start_pos: usize = reader.cur_pos;
        let count: usize = reader.read_usize()?;
        let mut locations: Vec<usize> = vec_with_capacity(count + 1)?;
        for _ in 0..count {
            let pointer: usize = reader.read_usize()?;
            if pointer != 0 {
                locations.push(pointer);
            }
        }
        locations.push(reader.chunk.end_pos);
        
        let shaders: Vec<GMShader> = Vec::with_capacity(count);
        for [pointer, next_pointer] in locations.windows(2) {
            reader.cur_pos = *pointer;
            let name: GMRef<String> = reader.read_gm_string()?;
            let shader_type: GMShaderType = num_enum_from(reader.read_u32()?)?; 
            
        }
        
        
        Ok(Self { shaders: global_init_scriptsfghdhsfhfs, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        if !self.exists { return Ok(()) }
        builder.write_simple_list_of_resource_ids(&self.shaders)?;
        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct GMShader {
}


/// Possible shader types a shader can have.
/// All console shaders (and HLSL11?) are compiled using confidential SDK tools when
/// GMAssetCompiler builds the game (for PSVita it's psp2cgc shader compiler).
#[derive(Debug, Clone, Copy, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum GMShaderType {
    GLSL_ES = 1,
    GLSL = 2,
    HLSL9 = 3,
    HLSL11 = 4,
    /// PSSL is a shading language used only in PS4, based on HLSL11.
    PSSL = 5,
    /// Cg stands for "C for graphics" made by NVIDIA and used in PSVita and PS3 (they have their own variants of Cg), based on HLSL9.
    Cg_PSVita = 6,
    /// Cg stands for "C for graphics" made by NVIDIA and used in PSVita and PS3 (they have their own variants of Cg), based on HLSL9.
    Cg_PS3 = 7
}

