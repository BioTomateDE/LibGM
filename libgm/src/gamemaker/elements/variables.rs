use std::ops::{Deref, DerefMut};

use crate::{
    gamemaker::{
        deserialize::{chunk::GMChunk, reader::DataReader},
        elements::{GMChunkElement, GMElement, general_info::GMGeneralInfo},
        reference::GMRef,
        serialize::{builder::DataBuilder, traits::GMSerializeIfVersion},
    },
    gml::instructions::GMInstanceType,
    prelude::*,
    util::init::{num_enum_from, vec_with_capacity},
};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GMVariables {
    /// List of all variables; mixing global, local and self.
    pub variables: Vec<GMVariable>,
    /// Set in bytecode 15 and above.
    pub b15_header: Option<GMVariablesB15Header>,
    pub exists: bool,
}

impl GMVariables {
    // TODO: make this work for bytecode 14. also docs. also vari_instance_type is wrong/buggy?
    pub fn make(
        &mut self,
        name: &str,
        instance_type: GMInstanceType,
        general_info: &GMGeneralInfo,
    ) -> Result<GMRef<GMVariable>> {
        // if general_info.bytecode_version < 15 {
        //     bail!("Bytecode 14 (and below) not yet supported");
        // }

        if instance_type == GMInstanceType::Local {
            bail!("Local variables have to be unique; this function will not work");
        }

        let vari_instance_type = if instance_type == GMInstanceType::Builtin {
            GMInstanceType::Self_
        } else {
            instance_type
        };

        for (i, variable) in self.variables.iter().enumerate() {
            if variable.name != name {
                continue;
            }

            if let Some(b15) = &variable.b15_data
                && b15.instance_type != vari_instance_type
            {
                continue;
            }

            // Found existing variable!
            return Ok(i.into());
        }

        // Couldn't find a variable; make a new one

        // First update these scuffed ass variable counts
        if let Some(b15_header) = &mut self.b15_header {
            //let mut variable_id: i32 = b15_header.var_count1 as i32;

            if general_info.is_version_at_least((2, 3)) {
                if instance_type != GMInstanceType::Builtin {
                    b15_header.var_count1 += 1;
                    b15_header.var_count2 += 1;
                    //variable_id = new_name_string.index as i32;
                }
            } else if general_info.bytecode_version >= 16 {
                // this condition is only suggested by utmt; not confirmed (original: `!DifferentVarCounts`)
                b15_header.var_count1 += 1;
                b15_header.var_count2 += 1;
            } else if instance_type == GMInstanceType::Self_ {
                //variable_id = b15_header.var_count2 as i32;
                b15_header.var_count2 += 1;
            } else if instance_type == GMInstanceType::Global {
                b15_header.var_count1 += 1;
            }
        }

        // if instance_type_VARI == GMInstanceType::Builtin {
        //     variable_id = -6;
        // }

        // Now actually create the variable
        let variable_ref: GMRef<GMVariable> = self.variables.len().into();

        let variable = GMVariable {
            name: name.to_string(),
            b15_data: Some(GMVariableB15Data {
                instance_type: vari_instance_type,
                variable_id: 0x6767,
            }),
        };

        self.variables.push(variable);

        Ok(variable_ref)
    }
}

impl Deref for GMVariables {
    type Target = Vec<GMVariable>;
    fn deref(&self) -> &Self::Target {
        &self.variables
    }
}

impl DerefMut for GMVariables {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.variables
    }
}

impl GMChunkElement for GMVariables {
    const NAME: &'static str = "VARI";
    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMElement for GMVariables {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let b15_header: Option<GMVariablesB15Header> =
            reader.deserialize_if_bytecode_version(15)?;
        let variable_size = match b15_header {
            Some(_) => 20,
            None => 12,
        };
        let variable_count = (reader.get_chunk_length() / variable_size) as usize;

        let mut occurrence_infos: Vec<(u32, u32)> = Vec::with_capacity(variable_count);
        let mut variables: Vec<GMVariable> = Vec::with_capacity(variable_count);

        // Parse variables
        while reader.cur_pos + variable_size <= reader.chunk.end_pos {
            let name: String = reader.read_gm_string()?;

            let b15_data: Option<GMVariableB15Data> = reader.deserialize_if_bytecode_version(15)?;

            let occurrence_count = reader.read_count("Variable occurrence")?;
            let first_occurrence_pos = reader.read_u32()?;
            occurrence_infos.push((occurrence_count, first_occurrence_pos));

            variables.push(GMVariable { name, b15_data });
        }

        // Resolve occurrences
        let saved_chunk: GMChunk = reader.chunk.clone();
        let saved_position = reader.cur_pos;
        reader.chunk = reader
            .chunks
            .get("CODE")
            .cloned()
            .ok_or("Chunk CODE not set while parsing variable occurrences")?;

        for (i, (occurrence_count, first_occurrence_pos)) in
            occurrence_infos.into_iter().enumerate()
        {
            let (occurrences, _name_string_id): (Vec<u32>, u32) =
                parse_occurrence_chain(reader, first_occurrence_pos, occurrence_count)?;

            // TODO: deal with the name string id somehow (also in FUNC)

            //  // Verify name string id.
            //  // Unused variables (`prototype`, `@@array@@` and all `arguments` in Undertale)
            //  // have a name string id of -1.
            //  if name_string_id as i32 != -1 && name.index != name_string_id {
            //      bail!(
            //          "Variable #{i} with name {:?} specifies name string id {}; but the id of name string is actually {}",
            //          reader.resolve_gm_str(name)?,
            //          name_string_id,
            //          name.index,
            //      );
            //  }

            for occurrence in occurrences {
                if let Some(old_value) = reader.variable_occurrences.insert(occurrence, i.into()) {
                    bail!(
                        "Conflicting occurrence positions while parsing variables: Position {} was already \
                        set for variable #{} with name {:?}; trying to set to variable #{i} with name {:?}",
                        occurrence,
                        old_value.index,
                        old_value.resolve(&variables)?.name,
                        variables[i].name,
                    );
                }
            }
        }

        reader.chunk = saved_chunk;
        reader.cur_pos = saved_position;

        Ok(GMVariables { variables, b15_header, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        self.b15_header
            .serialize_if_bytecode_ver(builder, "Scuffed bytecode 15 fields", 15)?;
        for (i, variable) in self.variables.iter().enumerate() {
            builder.write_gm_string(&variable.name);
            variable
                .b15_data
                .serialize_if_bytecode_ver(builder, "Bytecode 15 data", 15)?;

            let occurrences = builder.variable_occurrences.get(i).ok_or_else(|| {
                format!(
                    "Could not resolve variable occurrence with index {i} in list with length {}",
                    builder.function_occurrences.len()
                )
            })?;
            let occurrence_count: usize = occurrences.len();
            let first_occurrence: i32 = match occurrences.first() {
                Some((occurrence, _)) => *occurrence as i32,
                None => -1,
            };
            builder.write_usize(occurrence_count)?;
            builder.write_i32(first_occurrence);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMVariable {
    pub name: String,
    pub b15_data: Option<GMVariableB15Data>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMVariableB15Data {
    pub instance_type: GMInstanceType,
    pub variable_id: i32,
}

impl GMElement for GMVariableB15Data {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let instance_type = reader.read_i32()?;
        let instance_type: GMInstanceType = num_enum_from(instance_type)?;
        let variable_id = reader.read_i32()?;
        Ok(GMVariableB15Data { instance_type, variable_id })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i32(self.instance_type.into());
        builder.write_i32(self.variable_id);
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMVariablesB15Header {
    pub var_count1: u32,
    pub var_count2: u32,
    pub max_local_var_count: u32,
}

impl GMElement for GMVariablesB15Header {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        // Nobody knows what the fuck these values mean
        // TODO remember to increment these when a variable is added by a mod
        let var_count1 = reader.read_u32()?;
        let var_count2 = reader.read_u32()?;
        let max_local_var_count = reader.read_u32()?;
        Ok(GMVariablesB15Header {
            var_count1,
            var_count2,
            max_local_var_count,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_u32(self.var_count1);
        builder.write_u32(self.var_count2);
        builder.write_u32(self.max_local_var_count);
        Ok(())
    }
}

fn parse_occurrence_chain(
    reader: &mut DataReader,
    first_occurrence_pos: u32,
    occurrence_count: u32,
) -> Result<(Vec<u32>, u32)> {
    if occurrence_count < 1 {
        return Ok((vec![], first_occurrence_pos));
    }

    let mut occurrence_pos: u32 = first_occurrence_pos + 4;
    let mut occurrences: Vec<u32> = vec_with_capacity(occurrence_count)?;
    let mut offset: i32 = 6969; // Default value will never be relevant since it returns if no occurrences

    for _ in 0..occurrence_count {
        occurrences.push(occurrence_pos);
        reader.cur_pos = occurrence_pos;
        let raw_value = reader.read_i32()?;
        offset = raw_value & 0x07FF_FFFF;
        if offset < 1 {
            bail!(
                "Next occurrence offset is {0} (0x{0:08X}) which is negative while parsing \
                variable occurrences at position {1} (raw value is 0x{2:08X})",
                offset,
                reader.cur_pos - 4,
                raw_value,
            );
        }
        occurrence_pos += offset as u32; // Might overflow on last occurrence (name string id) but doesn't matter
    }

    let name_string_id: u32 = (offset & 0xFF_FFFF) as u32;
    Ok((occurrences, name_string_id))
}
