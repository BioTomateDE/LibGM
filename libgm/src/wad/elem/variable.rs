// SPDX-License-Identifier: GPL-3.0-only

use crate::gml::instruction::VariableType;
use crate::prelude::*;
use crate::wad::GMVersion;
use crate::wad::chunk::ChunkName;
use crate::wad::chunk::gm_list_chunk;
use crate::wad::elem::GMComplexElement;
use crate::wad::elem::GMElement;
use crate::wad::elem::GMNamedElement;
use crate::wad::elem::string::Strings;
use crate::wad::elem::validate_identifier;
use crate::wad::load::chunk::ChunkBounds;
use crate::wad::load::reader::DataReader;
use crate::wad::reference::GMRef;
use crate::wad::save::writer::DataWriter;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Variables {
    /// List of all variables; mixing global, local and self.
    pub elems: Vec<Variable>,

    /// Set in WAD 15 and above.
    pub modern_header: Option<ModernHeader>,
}

gm_list_chunk!(VARI, Variables, Variable);

impl GMElement for Variables {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let modern_header: Option<ModernHeader> =
            reader.deserialize_if_version(GMVersion::Wad15)?;
        let variable_size = match modern_header {
            Some(_) => 20,
            None => 12,
        };
        let remaining_length = reader.chunk.end_pos - reader.cur_pos;
        let variable_count = (remaining_length / variable_size) as usize;

        let mut elems: Vec<Variable> = Vec::with_capacity(variable_count);
        for idx in 0..variable_count {
            let variable = reader.read_elem_ctx(GMRef::from(idx))?;
            elems.push(variable);
        }

        Ok(Self { elems, modern_header })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_if_ver(&self.modern_header, "Scuffed WAD 15+ header", GMVersion::Wad15)?;
        for (idx, variable) in self.elems.iter().enumerate() {
            writer.write_elem_ctx(variable, idx)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    pub name: GMRef<String>,
    pub modern_data: Option<ModernVarData>,
}

impl GMComplexElement for Variable {
    type ReadContext = GMRef<Self>;
    type WriteContext = usize;

    fn deserialize_complex(reader: &mut DataReader, self_ref: GMRef<Self>) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;

        let modern_data: Option<ModernVarData> = reader.deserialize_if_version(GMVersion::Wad15)?;

        let occurrence_count = reader.read_count("Variable occurrence")?;
        let first_occurrence_pos = reader.read_u32()?;

        // placeholder value is never read in wad < 15
        let var_type = modern_data.as_ref().map_or(VariableType::Self_, |m| m.variable_type);
        read_occurrence_chain(reader, first_occurrence_pos, occurrence_count, self_ref, var_type)?;

        Ok(Self { name, modern_data })
    }

    fn serialize_complex(&self, writer: &mut DataWriter, self_idx: usize) -> Result<()> {
        let occurrences = &writer.variable_occurrences[self_idx];
        let occurrence_count = occurrences.len();
        let first_occurrence: i32 = match occurrences.first() {
            Some(&occurrence) => (occurrence & 0x07FF_FFFF) as i32,
            None => -1,
        };

        writer.write_gm_string(self.name)?;
        writer.write_if_ver(&self.modern_data, "WAD 15 data", GMVersion::Wad15)?;
        writer.write_usize(occurrence_count)?;
        writer.write_i32(first_occurrence);
        Ok(())
    }
}

impl GMNamedElement for Variable {
    fn name_ref(&self) -> GMRef<String> {
        self.name
    }

    fn validate_name(&self, gm_strings: &Strings) -> Result<()> {
        let name = self.name(gm_strings)?;
        if name == "$$$$temp$$$$" {
            return Ok(());
        }
        validate_identifier(name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModernVarData {
    pub variable_type: VariableType,
    pub variable_id: i32,
}

impl GMElement for ModernVarData {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let variable_type: i32 = reader.read_i32()?;
        let variable_type = VariableType::decode(variable_type)?;
        let variable_id = reader.read_i32()?;
        Ok(Self { variable_type, variable_id })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_i32(self.variable_type.encode());
        writer.write_i32(self.variable_id);
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
        Ok(Self { var_count1, var_count2, max_local_var_count })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_u32(self.var_count1);
        writer.write_u32(self.var_count2);
        writer.write_u32(self.max_local_var_count);
        Ok(())
    }
}

fn read_occurrence_chain(
    reader: &mut DataReader,
    first_occurrence_pos: u32,
    occurrence_count: u32,
    var_ref: GMRef<Variable>,
    var_type: VariableType,
) -> Result<()> {
    if occurrence_count < 1 {
        return Ok(());
    }

    let saved_chunk: ChunkBounds = reader.chunk;
    let saved_position = reader.cur_pos;
    reader.chunk = reader
        .chunks
        .get(ChunkName::CODE)
        .ok_or("Chunk CODE not set while parsing variable occurrences")?;

    reader.cur_pos = first_occurrence_pos + 4;

    for _ in 0..occurrence_count {
        if let Some(old_func) =
            reader.variable_occurrences.insert(reader.cur_pos, (var_ref, var_type))
        {
            bail!(
                "Conflicting variable occurrence position {} belongs to both {:?} and {:?}",
                reader.cur_pos,
                old_func,
                var_ref,
            )
        }

        let offset = reader.read_u32()? & 0x07FF_FFFF;
        reader.cur_pos -= 4;
        reader.cur_pos += offset;
    }

    reader.chunk = saved_chunk;
    reader.cur_pos = saved_position;
    Ok(())
}

impl Variables {
    /// Tries to find an existing variable with the specified name and variable type.
    ///
    /// Returns [`GMRef::none`] if no variable could be found or if the specified variable type is Local.
    #[must_use]
    pub fn find(
        &self,
        name: &str,
        variable_type: VariableType,
        strings: &Strings,
    ) -> GMRef<Variable> {
        if variable_type == VariableType::Local {
            return GMRef::none();
        }

        for (i, variable) in self.elems.iter().enumerate() {
            let Ok(var_name) = strings.by_ref(variable.name) else {
                continue;
            };
            if var_name != name {
                continue;
            }

            if let Some(modern) = &variable.modern_data
                && modern.variable_type != variable_type
            {
                continue;
            }

            // Found existing variable!
            return GMRef::from(i);
        }

        GMRef::none()
    }

    /// Creates a brand new variable with the specified variable type.
    ///
    /// TODO: does this affects correctness if variables aren't forced to be reused?
    ///
    /// This method is still buggy, use with caution.
    ///
    /// # Panics
    /// This functions panics when used with [`InstanceType::Local`].
    pub fn make_new(
        &mut self,
        name: GMRef<String>,
        variable_type: VariableType,
        version: GMVersion,
    ) -> GMRef<Variable> {
        assert!(
            variable_type != VariableType::Local,
            "Local variables have to be unique; this function will not work"
        );

        // First update these scuffed ass variable counts in WAD 15+
        let mut id = None;
        if let Some(header) = &mut self.modern_header {
            if version >= GMVersion::GMS2_3 {
                id = Some(name.as_raw());
                // NOTE: builtin is now handled differently because i split up
                // variable type and instance type. probably works riiiight

                // these counts are the same
                header.var_count1 += 1;
                header.var_count2 += 1;
            } else if version >= GMVersion::Wad16 {
                id = Some(header.var_count1 as i32);
                // these counts are the same
                header.var_count1 += 1;
                header.var_count2 += 1;
            } else if variable_type == VariableType::Self_ {
                id = Some(header.var_count2 as i32);
                header.var_count2 += 1;
            } else if variable_type == VariableType::Global {
                id = Some(header.var_count1 as i32);
                header.var_count1 += 1;
            }
        }

        // Now actually create the variable
        let modern_data = id.map(|variable_id| ModernVarData { variable_type, variable_id });
        let variable = Variable { name, modern_data };
        self.push(variable)
    }

    /// jank
    pub fn make(
        &mut self,
        name: &str,
        variable_type: VariableType,
        version: GMVersion,
        strings: &mut Strings,
    ) -> GMRef<Variable> {
        let existing = self.find(name, variable_type, strings);
        if existing.is_some() {
            return existing;
        }
        self.make_new(strings.make(name), variable_type, version)
    }

    pub fn validate_names(&self, strings: &Strings) -> Result<()> {
        for var in &self.elems {
            let name: &str = var.name(strings)?;
            var.validate_name(strings).ctx(|| format!("validating Variable with name {name:?}"))?;
        }
        Ok(())
    }
}
