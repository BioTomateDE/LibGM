use itertools::izip;
use crate::deserialize::chunk_reading::{GMChunk, GMRef};
use crate::deserialize::code::{parse_instance_type, GMCodeVariable, GMDataType, GMInstanceType, GMOpcode, GMPopInstruction, GMVariableType};
use crate::deserialize::general_info::GMGeneralInfo;
use crate::deserialize::strings::GMStrings;

#[derive(Debug, Clone)]
pub struct GMVariable {
    pub name: GMRef<String>,
    pub instance_type: GMInstanceType,
    pub variable_id: Option<i32>,
    /// List of occurrences of the variable in code as absolute positions. Only meant for parsing code; irrelevant (and potentially incorrect) after parsing.
    pub occurrences: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct GMVariables {
    pub global_variables: Vec<GMVariable>,
    pub instance_variables: Vec<GMVariable>,
    pub local_variables: Vec<GMVariable>,
}

pub fn parse_chunk_vari(chunk: &mut GMChunk, strings: &GMStrings, general_info: &GMGeneralInfo, chunk_code: &mut GMChunk) -> Result<GMVariables, String> {
    // TODO please someone fix this. also in parsing chunk CODE.
    chunk.cur_pos = 0;
    let globals_count: usize = chunk.read_usize()?;         // the amount of global variables
    let instances_count: usize = chunk.read_usize()?;       // the amount of `Self` variables (local to own object)
    let locals_count: usize = chunk.read_usize()?;          // the amount of `Local` variables (local to own code script)

    let mut global_variables: Vec<GMVariable> = Vec::with_capacity(globals_count);
    let mut instance_variables: Vec<GMVariable> = Vec::with_capacity(instances_count);
    let mut local_variables: Vec<GMVariable> = Vec::with_capacity(locals_count);

    for (variable_count, variables, default_instance_type) in izip!(
        [globals_count, instances_count, locals_count],
        [&mut global_variables, &mut instance_variables, &mut local_variables],
        [GMInstanceType::Global, GMInstanceType::Self_(None), GMInstanceType::Local],
    ) {
        for i in 0..variable_count {
            let name: GMRef<String> = chunk.read_gm_string(strings)?;

            // bytecode>=15 might reads instance type here, so maybe it doesn't have the 3 global/instance/local count in the beginning?
            let mut instance_type: GMInstanceType = default_instance_type.clone();
            let mut variable_id: Option<i32> = None;
            if general_info.bytecode_version >= 15 {
                instance_type = parse_instance_type(chunk.read_i32()? as i16)
                    .map_err(|e| format!("Could not get instance type for variable \"{}\" while parsing chunk VARI: {e}", name.display(strings)))?;
                // let instance_type_: i32 = chunk.read_i32()?;
                // instance_type = instance_type_.try_into()
                //     .map_err(|_| format!("Invalid instance type 0x{instance_type_:8X} at absolute position {} while parsing variables.", chunk.cur_pos+chunk.abs_pos))?;
                variable_id = Some(chunk.read_i32()?);
            }

            let occurrences_count: usize = chunk.read_usize()?;
            let first_occurrence_address: i32 = chunk.read_i32()?;

            let occurrences: Vec<usize> = parse_occurrence_chain(
                chunk_code,
                name.display(strings),
                GMRef::new(i),
                first_occurrence_address,
                occurrences_count
            )?;

            variables.push(GMVariable {
                name,
                instance_type,
                variable_id,
                occurrences,
            });
        }
    }

    // log::debug!("var len: {}", variables_by_index.len());
    Ok(GMVariables {
        global_variables,
        instance_variables,
        local_variables,
    })
}


fn parse_occurrence_chain(
    chunk_code: &mut GMChunk,
    variable_name: &str,
    variable_ref: GMRef<GMVariable>,
    first_occurrence_abs_pos: i32,
    occurrence_count: usize,
) -> Result<Vec<usize>, String> {
    if occurrence_count < 1 {
        return Ok(vec![]);
    }

    let occurrence_pos: i32 = first_occurrence_abs_pos - chunk_code.abs_pos as i32;
    let mut occurrence_pos: usize = occurrence_pos.try_into()
        .map_err(|_| format!(
            "First occurrence of variable \"{}\" is out of bounds; should be: {} <= {} < {}.",
            variable_name, chunk_code.abs_pos, first_occurrence_abs_pos, chunk_code.abs_pos + chunk_code.data.len(),
        ))?;

    let mut occurrences: Vec<usize> = Vec::with_capacity(occurrence_count);

    for _ in 0..occurrence_count {
        chunk_code.cur_pos = occurrence_pos;
        let offset = read_variable_reference(chunk_code, variable_ref.clone())?;
        occurrence_pos += offset;
        occurrences.push(occurrence_pos + chunk_code.abs_pos);
    }

    Ok(occurrences)
}


pub fn read_variable_reference(chunk: &mut GMChunk, variable: GMRef<GMVariable>) -> Result<usize, String> {
    // let b0: u8 = chunk.read_u8()?;
    // let b1: u8 = chunk.read_u8()?;
    // let b2: u8 = chunk.read_u8()?;
    // let raw_opcode: u8 = chunk.read_u8()?;
    chunk.cur_pos += 4;  // skip ^
    let raw_value: i32 = chunk.read_i32()?;

    // if bytecode14 {
    //     raw_opcode = convert_instruction_kind(raw_opcode);
    // }
    // let opcode: GMOpcode = raw_opcode.try_into()
    //     .map_err(|_| format!("Invalid Opcode 0x{raw_opcode:02X} while parsing code instruction."))?;

    // match opcode {
    //     GMOpcode::Pop | GMOpcode::Popz | GMOpcode::PopEnv => {
    //         let type1: u8 = b2 & 0xf;
    //         let type1: GMDataType = type1.try_into()
    //             .map_err(|_| format!("Invalid Data Type 1 {type1:02X} while parsing Pop Instruction for variable reference chain."))?;
    //
    //         let type2: u8 = b2 >> 4;
    //         let type2: GMDataType = type2.try_into()
    //             .map_err(|_| format!("Invalid Data Type 2 {type2:02X} while parsing Pop Instruction for variable reference chain."))?;
    //
    //         let instance_type: i16 = b0 as i16 | ((b1 as i16) << 8);
    //         let instance_type: GMInstanceType = parse_instance_type(instance_type)?;
    //
    //         GMPopInstruction
    //     }
    //
    //     GMOpcode::Push | GMOpcode::PushEnv | GMOpcode::PushBltn | GMOpcode::PushGlb | GMOpcode::PushLoc => {
    //
    //     }
    //
    //     other => return Err(format!("Invalid opcode {other:?} while parsing reference chain of variable."))
    // }

    // let variable_type: i32 = (raw_value >> 24) & 0xF8;
    // let variable_type: u8 = variable_type as u8;
    // let variable_type: GMVariableType = variable_type.try_into()
    //     .map_err(|_| format!("Invalid Variable Type 0x{variable_type:02X} while parsing variable reference chain."))?;

    let next_occurrence_offset: i32 = raw_value & 0x07FFFFFF;
    let next_occurrence_offset: usize = next_occurrence_offset as usize;


    // let destination = GMCodeVariable {
    //     variable,
    //     variable_type
    // };

    // let instruction = GMPopInstruction {
    //     opcode,
    //     instance_type,
    //     type1,
    //     type2,
    //     destination,
    // };

    Ok(next_occurrence_offset)
}

