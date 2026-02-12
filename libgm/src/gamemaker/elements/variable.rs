use macros::list_chunk;

use crate::{
    gamemaker::{
        deserialize::{chunk::ChunkBounds, reader::DataReader},
        elements::{
            GMElement, GMNamedElement, element_stub, general_info::GMGeneralInfo,
            validate_identifier,
        },
        reference::GMRef,
        serialize::builder::DataBuilder,
    },
    gml::instruction::InstanceType,
    prelude::*,
    util::init::vec_with_capacity,
};

#[list_chunk("VARI")]
pub struct GMVariables {
    /// List of all variables; mixing global, local and self.
    pub variables: Vec<GMVariable>,
    /// Set in WAD 15 and above.
    pub modern_header: Option<ModernHeader>,
    pub exists: bool,
}

impl GMVariables {
    // This method is still buggy, use with caution.
    // TODO: make this work for WAD<=14. also docs. also vari_instance_type is wrong/buggy?
    pub fn make(
        &mut self,
        name: &str,
        instance_type: InstanceType,
        general_info: &GMGeneralInfo,
    ) -> Result<GMRef<GMVariable>> {
        if instance_type == InstanceType::Local {
            bail!("Local variables have to be unique; this function will not work");
        }

        let vari_instance_type = if instance_type == InstanceType::Builtin {
            InstanceType::Self_
        } else {
            instance_type
        };

        for (i, variable) in self.variables.iter().enumerate() {
            if variable.name != name {
                continue;
            }

            if let Some(data) = &variable.modern_data
                && data.instance_type != vari_instance_type
            {
                continue;
            }

            // Found existing variable!
            return Ok(i.into());
        }

        // Couldn't find a variable; make a new one

        // First update these scuffed ass variable counts
        if let Some(header) = &mut self.modern_header {
            if general_info.is_version_at_least((2, 3)) {
                if instance_type != InstanceType::Builtin {
                    header.var_count1 += 1;
                    header.var_count2 += 1;
                }
            } else if general_info.wad_version >= 16 {
                // this condition is only suggested by utmt; not confirmed (original: `!DifferentVarCounts`)
                header.var_count1 += 1;
                header.var_count2 += 1;
            } else if matches!(
                instance_type,
                InstanceType::Self_ | InstanceType::GameObject(_)
            ) {
                header.var_count2 += 1;
            } else if instance_type == InstanceType::Global {
                header.var_count1 += 1;
            }
        }

        // Now actually create the variable
        let variable_ref: GMRef<GMVariable> = self.variables.len().into();

        let variable = GMVariable {
            name: name.to_string(),
            modern_data: Some(ModernData {
                instance_type: vari_instance_type,
                variable_id: 0x6767,
            }),
        };

        self.variables.push(variable);

        Ok(variable_ref)
    }
}

impl GMElement for GMVariables {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let modern_header: Option<ModernHeader> = reader.deserialize_if_wad_version(15)?;
        let variable_size = match modern_header {
            Some(_) => 20,
            None => 12,
        };
        let variable_count = (reader.chunk.length() / variable_size) as usize;

        let mut occurrence_infos: Vec<(u32, u32)> = Vec::with_capacity(variable_count);
        let mut variables: Vec<GMVariable> = Vec::with_capacity(variable_count);

        // Parse variables
        while reader.cur_pos + variable_size <= reader.chunk.end_pos {
            let name: String = reader.read_gm_string()?;

            let modern_data: Option<ModernData> = reader.deserialize_if_wad_version(15)?;

            let occurrence_count = reader.read_count("Variable occurrence")?;
            let first_occurrence_pos = reader.read_u32()?;
            occurrence_infos.push((occurrence_count, first_occurrence_pos));

            variables.push(GMVariable { name, modern_data });
        }

        // Resolve occurrences
        let saved_chunk: ChunkBounds = reader.chunk.clone();
        let saved_position = reader.cur_pos;
        reader.chunk = reader
            .chunks
            .get("CODE")
            .ok_or("Chunk CODE not set while parsing variable occurrences")?;

        for (i, (occurrence_count, first_occurrence_pos)) in
            occurrence_infos.into_iter().enumerate()
        {
            let occurrences: Vec<u32> =
                parse_occurrence_chain(reader, first_occurrence_pos, occurrence_count)?;

            // TODO: this code is extremely ugly.
            // the hashmaps are probably also slow.
            // refactor this entire thing
            for occurrence in occurrences {
                let var_ref: GMRef<GMVariable> = i.into();
                // fallback value is never read in wad < 15
                let instance_type = variables[i]
                    .modern_data
                    .as_ref()
                    .map_or(InstanceType::Self_, |d| d.instance_type);

                if let Some(old_var) = reader
                    .variable_occurrences
                    .insert(occurrence, (var_ref, instance_type))
                {
                    bail!(
                        "Conflicting occurrence positions while parsing variables: Position {} was already \
                        set for variable #{} with name {:?}; trying to set to variable #{i} with name {:?}",
                        occurrence,
                        old_var.0.index,
                        variables[old_var.0.index as usize].name,
                        variables[i].name,
                    );
                }
            }
        }

        reader.chunk = saved_chunk;
        reader.cur_pos = saved_position;

        Ok(Self { variables, modern_header, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_if_wad_ver(&self.modern_header, "Scuffed WAD 15+ fields", 15)?;
        for (i, variable) in self.variables.iter().enumerate() {
            builder.write_gm_string(&variable.name);
            builder.write_if_wad_ver(&variable.modern_data, "WAD 15 data", 15)?;

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
    pub modern_data: Option<ModernData>,
}

element_stub!(GMVariable);
impl GMNamedElement for GMVariable {
    fn name(&self) -> &String {
        &self.name
    }

    fn name_mut(&mut self) -> &mut String {
        &mut self.name
    }

    fn validate_name(&self) -> Result<()> {
        if self.name == "$$$$temp$$$$" {
            return Ok(());
        }
        validate_identifier(&self.name)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModernData {
    pub instance_type: InstanceType,
    pub variable_id: i32,
}

impl GMElement for ModernData {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let raw_instance_type: i16 = reader.read_i32()? as i16;
        let instance_type: InstanceType = InstanceType::parse_normal(raw_instance_type)?;
        let variable_id = reader.read_i32()?;
        Ok(Self { instance_type, variable_id })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_i32(i32::from(self.instance_type.build()));
        builder.write_i32(self.variable_id);
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModernHeader {
    pub var_count1: u32,
    pub var_count2: u32,
    pub max_local_var_count: u32,
}

impl GMElement for ModernHeader {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        // Nobody knows what the fuck these values mean
        let var_count1 = reader.read_u32()?;
        let var_count2 = reader.read_u32()?;
        let max_local_var_count = reader.read_u32()?;
        Ok(Self {
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
) -> Result<Vec<u32>> {
    if occurrence_count < 1 {
        return Ok(vec![]);
    }

    let mut occurrence_pos: u32 = first_occurrence_pos + 4;
    let mut occurrences: Vec<u32> = vec_with_capacity(occurrence_count)?;
    let mut offset: i32;

    for i in 0..occurrence_count {
        occurrences.push(occurrence_pos);
        reader.cur_pos = occurrence_pos;
        let raw_value = reader.read_i32()?;
        offset = raw_value & 0x07FF_FFFF;

        if offset < 1 {
            bail!(
                "Next occurrence offset is {offset} (0x{offset:08X}) which is \
                not positive for variable occurrence {i}/{occurrence_count}"
            );
        }

        occurrence_pos += offset as u32;
    }

    Ok(occurrences)
}
