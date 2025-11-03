use crate::gamemaker::deserialize::chunk::GMChunk;
use crate::gamemaker::deserialize::reader::DataReader;
use crate::gamemaker::deserialize::resources::GMRef;
use crate::gamemaker::elements::code::{GMInstanceType, GMVariableType, build_instance_type, parse_instance_type};
use crate::gamemaker::elements::strings::GMStrings;
use crate::gamemaker::elements::{GMChunkElement, GMElement};
use crate::gamemaker::serialize::builder::DataBuilder;
use crate::gamemaker::serialize::traits::GMSerializeIfVersion;
use crate::prelude::*;
use crate::util::init::vec_with_capacity;

#[derive(Debug, Clone, Default)]
pub struct GMVariables {
    /// List of all variables; mixing global, local and self.
    pub variables: Vec<GMVariable>,
    /// Set in bytecode 15 and above.
    pub b15_header: Option<GMVariablesB15Header>,
    pub exists: bool,
}

impl GMChunkElement for GMVariables {
    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMElement for GMVariables {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        if reader.get_chunk_length() == 0 {
            return Ok(Self { variables: vec![], b15_header: None, exists: true });
        }
        let variables_length = if reader.general_info.bytecode_version >= 15 {
            20
        } else {
            12
        };
        let variable_count = (reader.get_chunk_length() / variables_length) as usize;
        let b15_header: Option<GMVariablesB15Header> = reader.deserialize_if_bytecode_version(15)?;

        let mut occurrence_infos: Vec<(u32, u32)> = Vec::with_capacity(variable_count);
        let mut variables: Vec<GMVariable> = Vec::with_capacity(variable_count);

        // Parse variables
        while reader.cur_pos + variables_length <= reader.chunk.end_pos {
            let name: GMRef<String> = reader.read_gm_string()?;

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
            .context("Chunk CODE not set while parsing variable occurrences")?;

        for (i, (occurrence_count, first_occurrence_pos)) in occurrence_infos.into_iter().enumerate() {
            let name: GMRef<String> = variables[i].name;
            let (occurrences, name_string_id): (Vec<u32>, u32) =
                parse_occurrence_chain(reader, first_occurrence_pos, occurrence_count)?;

            // Verify name string id. unused variables (`prototype`, `@@array@@` and all
            // `arguments` in ut) have a name string id of -1.
            if name_string_id as i32 != -1 && name.index != name_string_id {
                bail!(
                    "Variable #{i} with name {:?} specifies name string id {}; but the id of name string is actually {}",
                    reader.resolve_gm_str(name)?,
                    name_string_id,
                    name.index,
                );
            }

            for occurrence in occurrences {
                if let Some(old_value) = reader.variable_occurrences.insert(occurrence, GMRef::new(i as u32)) {
                    bail!(
                        "Conflicting occurrence positions while parsing variables: Position {} was already \
                        set for variable #{} with name {:?}; trying to set to variable #{i} with name {:?}",
                        occurrence,
                        old_value.index,
                        reader.resolve_gm_str(old_value.resolve(&variables)?.name)?,
                        reader.resolve_gm_str(variables[i].name)?,
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
            builder.write_gm_string(&variable.name)?;
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

impl GMVariables {
    pub fn get_variable_ref_by_name(&self, name: &str, gm_strings: &GMStrings) -> Result<GMRef<GMVariable>> {
        for (i, variable) in self.variables.iter().enumerate() {
            let variable_name: &String = variable.name.resolve(&gm_strings.strings)?;
            if variable_name == name {
                return Ok(GMRef::new(i as u32));
            }
        }
        bail!("Could not resolve variable with name {name:?}");
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMVariable {
    pub name: GMRef<String>,
    pub b15_data: Option<GMVariableB15Data>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMVariableB15Data {
    pub instance_type: GMInstanceType,
    pub variable_id: i32,
}
impl GMElement for GMVariableB15Data {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let raw_instance_type: i16 = reader.read_i32()? as i16;
        let instance_type: GMInstanceType = parse_instance_type(raw_instance_type, GMVariableType::Normal)?;
        let variable_id = reader.read_i32()?;
        Ok(GMVariableB15Data { instance_type, variable_id })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i32(build_instance_type(&self.instance_type) as i32);
        builder.write_i32(self.variable_id);
        Ok(())
    }
}

#[derive(Debug, Clone)]
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
        Ok(GMVariablesB15Header { var_count1, var_count2, max_local_var_count })
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
        offset = raw_value & 0x07FFFFFF;
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

    let name_string_id: u32 = (offset & 0xFFFFFF) as u32;
    Ok((occurrences, name_string_id))
}

pub fn to_vari_instance_type(instance_type: &GMInstanceType) -> GMInstanceType {
    match instance_type {
        GMInstanceType::StackTop => GMInstanceType::Self_(None),
        GMInstanceType::Builtin => GMInstanceType::Self_(None),
        GMInstanceType::Self_(Some(_)) => GMInstanceType::Self_(None),
        GMInstanceType::Argument => GMInstanceType::Builtin,
        GMInstanceType::Other => GMInstanceType::Self_(None),
        _ => instance_type.clone(),
    }
}
