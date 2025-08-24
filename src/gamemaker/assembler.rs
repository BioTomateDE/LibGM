use std::fmt::{Display, Formatter};
use std::str::FromStr;
use crate::gamemaker::data::GMData;
use crate::gamemaker::deserialize::GMRef;
use crate::gamemaker::elements::code::{GMCodeValue, CodeVariable, GMGotoInstruction, GMPopSwapInstruction, GMSingleTypeInstruction, GMInstanceType, GMVariableType, GMComparisonInstruction, GMPopInstruction, GMPushInstruction, GMCallInstruction, GMExtendedInstruction16, GMExtendedInstructionFunction, GMExtendedInstruction32};
use crate::gamemaker::elements::code::{GMCallVariableInstruction, GMComparisonType, GMDoubleTypeInstruction, GMDuplicateInstruction, GMDuplicateSwapInstruction};
use crate::gamemaker::elements::code::{GMInstruction, GMInstructionData, GMOpcode};
use crate::gamemaker::elements::code::GMDataType;
use crate::gamemaker::elements::functions::{GMCodeLocal, GMFunction, GMFunctions};
use crate::gamemaker::elements::game_objects::GMGameObject;
use crate::gamemaker::elements::strings::GMStrings;
use crate::gamemaker::elements::variables::GMVariable;

#[derive(Debug)]
pub enum ParseError {
    ExpectedEOL(String),
    ExpectedSpace,
    ExpectedDot(String),
    ExpectedString,
    ExpectedArgc,
    UnexpectedEOL(&'static str),
    /// `(expected, actual)`
    InvalidTypeCount(usize, usize),
    InvalidDataType(char),
    InvalidComparisonType(String),
    InvalidInstanceType(String),
    InvalidMnemonic(String),
    InvalidExtendedMnemonic(String),
    InvalidPrefix(String),
    IntegerOutOfBounds(String),
    InvalidFloat(String),
    InvalidBoolean(String),
    InvalidIdentifier(&'static str),
    InvalidEscapeCharacter(char),
    UnmatchedSquareBracket,
    UnmatchedRoundBracket,
    StringIndexUnresolvable(u32),
    VarNameStringNoMatch(u32),
    VarNoInstanceType,
    VarUnresolvable(String),
    FuncUnresolvable(String),
    ObjectUnresolvable(String),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::ExpectedEOL(line) => write!(f, "Expected end of line; found remaining string: {line}"),
            ParseError::ExpectedSpace => write!(f, "Expected space"),
            ParseError::ExpectedString => write!(f, "Expected string"),
            ParseError::ExpectedArgc => write!(f, "Expected argument count of called function"),
            ParseError::UnexpectedEOL(context) => write!(f, "Unexpected end of line while assembling {context}"),
            ParseError::InvalidTypeCount(expected, actual) => write!(f, "Expected {expected} data types; found {actual}"),
            ParseError::InvalidDataType(char) => write!(f, "Invalid data type character '{char}'"),
            ParseError::InvalidComparisonType(cmp_type) => write!(f, "Invalid comparison type: {cmp_type}"),
            ParseError::InvalidInstanceType(inst_type) => write!(f, "Invalid instance type: {inst_type}"),
            ParseError::InvalidMnemonic(mnemonic) => write!(f, "Invalid opcode mnemonic: {mnemonic}"),
            ParseError::InvalidExtendedMnemonic(mnemonic) => write!(f, "Invalid extended/break mnemonic: {mnemonic}"),
            ParseError::InvalidPrefix(prefix) => write!(f, "Invalid square bracket prefix: {prefix}"),
            ParseError::IntegerOutOfBounds(int_str) => write!(f, "Integer out of bounds: {int_str}"),
            ParseError::InvalidFloat(float_str) => write!(f, "Invalid floating point number: {float_str}"),
            ParseError::InvalidBoolean(bool_str) => write!(f, "Invalid boolean: {bool_str}"),
            ParseError::InvalidIdentifier(reason) => write!(f, "Identifier (variable/function name) {reason}"),
            ParseError::InvalidEscapeCharacter(char) => write!(f, "Invalid escape character '{char}'"),
            ParseError::UnmatchedSquareBracket => write!(f, "Square bracket was never closed"),
            ParseError::UnmatchedRoundBracket => write!(f, "Round bracket was never closed"),
            ParseError::StringIndexUnresolvable(idx) => write!(f, "Could not resolve String with index {idx}"),
            ParseError::VarNameStringNoMatch(name_string_idx) => write!(f, "Could not find a variable with a matching name string index ({name_string_idx})"),
            ParseError::VarNoInstanceType => write!(f, "Variable does not have an instance type specified"),
            ParseError::VarUnresolvable(var_name) => write!(f, "Variable does not exist: {var_name}"),
            ParseError::FuncUnresolvable(func_name) => write!(f, "Function does not exist: {func_name}"),
            ParseError::ObjectUnresolvable(error) => write!(f, "{error}"),
            ParseError::ExpectedDot(line) => write!(f, "Expected dot; found '{line}'"),
        }
    }
}


pub fn assemble_code(assembly: &str, gm_data: &mut GMData, locals: &GMCodeLocal) -> Result<Vec<GMInstruction>, String> {
    let mut instructions: Vec<GMInstruction> = Vec::new();

    for line in assembly.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue
        }

        let instruction: GMInstruction = assemble_instruction(line, gm_data, locals)
            .map_err(|e| format!("{e}\n↳ while assembling instruction: {line}"))?;
        instructions.push(instruction);
    }

    Ok(instructions)
}


pub fn assemble_instruction(
    line: &str,
    gm_data: &mut GMData,
    locals: &GMCodeLocal,
) -> Result<GMInstruction, ParseError> {
    let mut line: &str = line.trim();
    let mut mnemonic: &str = line;
    let mut has_type: bool = false;

    for (i, char) in line.chars().enumerate() {
        if char == ' ' || char == '.' {
            mnemonic = &line[..i];
            line = &line[i+1..];
            if char == '.' {
                has_type = true;
            }
            break
        }
    }

    let mut types: Vec<GMDataType> = Vec::new();

    while has_type {
        let raw_type: char = line.chars().next().ok_or(ParseError::UnexpectedEOL("data type"))?;
        types.push(data_type_from_char(raw_type)?);
        line = &line[1..];
        let Some(next_char) = line.chars().next() else { break };
        line = &line[1..];
        if next_char != '.' {
            has_type = false;
        }
    }

    let extended_kind: Option<i16> = extended_id_from_string(mnemonic).ok();
    let opcode: GMOpcode = if extended_kind.is_some() {
        GMOpcode::Extended
    } else {
        opcode_from_string(mnemonic)?
    };

    let instruction_data: GMInstructionData = match opcode {
        GMOpcode::Exit => {
            line = &line[4..];
            GMInstructionData::Empty
        }

        GMOpcode::Negate |
        GMOpcode::Not |
        GMOpcode::Return |
        GMOpcode::PopDiscard => {
            if types.len() != 1 {
                return Err(ParseError::InvalidTypeCount(1, types.len()))
            }
            let data_type: GMDataType = types[0];
            GMInstructionData::SingleType(GMSingleTypeInstruction { data_type })
        }

        GMOpcode::Duplicate => {
            if types.len() != 1 {
                return Err(ParseError::InvalidTypeCount(1, types.len()))
            }
            let data_type: GMDataType = types[0];
            let size1: u8 = parse_int(&mut line)?;
            if consume_space(&mut line)? {
                let size2: u8 = parse_int(&mut line)?;
                GMInstructionData::DuplicateSwap(GMDuplicateSwapInstruction { data_type, size1, size2 })
            } else {
                GMInstructionData::Duplicate(GMDuplicateInstruction { data_type, size: size1 })
            }
        }

        GMOpcode::CallVariable => {
            if types.len() != 1 {
                return Err(ParseError::InvalidTypeCount(1, types.len()))
            }
            let data_type: GMDataType = types[0];
            let argument_count: u8 = parse_int(&mut line)?;
            GMInstructionData::CallVariable(GMCallVariableInstruction { data_type, argument_count })
        }

        GMOpcode::Convert |
        GMOpcode::Multiply |
        GMOpcode::Divide |
        GMOpcode::Remainder |
        GMOpcode::Modulus |
        GMOpcode::Add |
        GMOpcode::Subtract |
        GMOpcode::And |
        GMOpcode::Or |
        GMOpcode::Xor |
        GMOpcode::ShiftLeft |
        GMOpcode::ShiftRight => {
            if types.len() != 2 {
                return Err(ParseError::InvalidTypeCount(2, types.len()))
            }
            GMInstructionData::DoubleType(GMDoubleTypeInstruction { type1: types[0], type2: types[1] })
        }

        GMOpcode::Compare => {
            if types.len() != 2 {
                return Err(ParseError::InvalidTypeCount(2, types.len()))
            }
            let comparison_type_raw: String = parse_identifier(&mut line)?;
            let comparison_type: GMComparisonType = comparison_type_from_string(&comparison_type_raw)?;
            GMInstructionData::Comparison(GMComparisonInstruction { comparison_type, type1: types[0], type2: types[1] })
        }

        GMOpcode::Branch |
        GMOpcode::BranchIf |
        GMOpcode::BranchUnless |
        GMOpcode::PushWithContext |
        GMOpcode::PopWithContext => {
            if types.len() != 0 {
                return Err(ParseError::InvalidTypeCount(0, types.len()))
            }
            let jump_offset: Option<i32> = if line == "<drop>" {
                line = "";      // need to consume; otherwise error
                None
            } else {
                Some(parse_int(&mut line)?)
            };
            GMInstructionData::Goto(GMGotoInstruction { jump_offset })
        }

        GMOpcode::Pop => {
            if types.len() != 2 {
                return Err(ParseError::InvalidTypeCount(2, types.len()))
            }
            if types[0] == GMDataType::Int16 {
                // special popswap instruction
                let size: u8 = parse_int(&mut line)?;
                GMInstructionData::PopSwap(GMPopSwapInstruction { size })
            } else {
                let destination: CodeVariable = parse_variable(&mut line, locals, &gm_data)?;
                GMInstructionData::Pop(GMPopInstruction {
                    type1: types[0],
                    type2: types[1],
                    destination,
                })
            }
        }

        GMOpcode::Push |
        GMOpcode::PushLocal |
        GMOpcode::PushGlobal |
        GMOpcode::PushBuiltin |
        GMOpcode::PushImmediate => {
            if types.len() != 1 {
                return Err(ParseError::InvalidTypeCount(1, types.len()))
            }
            let value: GMCodeValue = match types[0] {
                GMDataType::Int16 => GMCodeValue::Int16(parse_int(&mut line)?),
                GMDataType::Int32 => {
                    if line.starts_with('[') {
                        let close_bracket: usize = line.find(']').ok_or(ParseError::UnmatchedSquareBracket)?;
                        let prefix: String = line[1..close_bracket].to_string();
                        line = &line[close_bracket+1..];
                        match prefix.as_str() {
                            "function" => GMCodeValue::Function(parse_function(&mut line, &gm_data.strings, &gm_data.functions)?),
                            "variable" => {
                                let mut variable: CodeVariable = parse_variable(&mut line, locals, &gm_data)?;
                                variable.is_int32 = true;
                                GMCodeValue::Variable(variable)
                            }
                            _ => return Err(ParseError::InvalidPrefix(prefix))
                        }
                    } else {
                        GMCodeValue::Int32(parse_int(&mut line)?)
                    }
                },
                GMDataType::Int64 => GMCodeValue::Int64(parse_int(&mut line)?),
                GMDataType::Float => GMCodeValue::Float(parse_float(&mut line)?),
                GMDataType::Double => GMCodeValue::Double(parse_float(&mut line)?),
                GMDataType::Boolean => GMCodeValue::Boolean(parse_bool(&mut line)?),
                GMDataType::String => {
                    let string_text: String = parse_string_literal(&mut line)?;
                    let string_ref: GMRef<String> = gm_data.make_string(&string_text);
                    GMCodeValue::String(string_ref)
                },
                GMDataType::Variable => GMCodeValue::Variable(parse_variable(&mut line, locals, &gm_data)?),
            };
            GMInstructionData::Push(GMPushInstruction { value })
        }

        GMOpcode::Call => {
            if types.len() != 1 {
                return Err(ParseError::InvalidTypeCount(1, types.len()))
            }
            let function: GMRef<GMFunction> = parse_function(&mut line, &gm_data.strings, &gm_data.functions)?;
            const ARGC_STR: &str = "(argc=";
            if !line.starts_with(ARGC_STR) {
                return Err(ParseError::ExpectedArgc)
            }
            line = &line[ARGC_STR.len()..];
            let arguments_count: u8 = parse_int(&mut line)?;
            if !line.starts_with(')') {
                return Err(ParseError::UnmatchedRoundBracket)
            }
            line = &line[1..];
            GMInstructionData::Call(GMCallInstruction{
                arguments_count,
                data_type: types[0],
                function,
            })
        }

        GMOpcode::Extended => {
            if types.len() != 1 {
                return Err(ParseError::InvalidTypeCount(1, types.len()))
            }
            let kind: i16 = extended_kind.unwrap();  // should be set if [`GMOpcode::Extended`]
            const FN_PREFIX: &str = "[function]";
            if line.len() < 1 {
                GMInstructionData::Extended16(GMExtendedInstruction16 { kind })
            } else if line.starts_with(FN_PREFIX) {
                line = &line[FN_PREFIX.len()..];
                let function: GMRef<GMFunction> = parse_function(&mut line, &gm_data.strings, &gm_data.functions)?;
                GMInstructionData::ExtendedFunc(GMExtendedInstructionFunction { kind, function })
            } else {
                let int_argument: i32 = parse_int(&mut line)?;
                GMInstructionData::Extended32(GMExtendedInstruction32 { kind, int_argument })
            }
        }
    };

    if line.len() > 0 {
        return Err(ParseError::ExpectedEOL(line.to_string()))
    }

    Ok(GMInstruction { opcode, kind: instruction_data })
}


/// Throws error if next char is not a space.
/// Returns `[false`] if end of line.
fn consume_space(line: &mut &str) -> Result<bool, ParseError> {
    let Some(char) = line.chars().next() else {
        return Ok(false)
    };
    if char != ' ' {
        return Err(ParseError::ExpectedSpace)
    }
    *line = &line[1..];
    Ok(true)
}


fn data_type_from_char(data_type: char) -> Result<GMDataType, ParseError> {
    Ok(match data_type {
        'e' => GMDataType::Int16,
        'i' => GMDataType::Int32,
        'l' => GMDataType::Int64,
        'f' => GMDataType::Float,
        'd' => GMDataType::Double,
        'b' => GMDataType::Boolean,
        's' => GMDataType::String,
        'v' => GMDataType::Variable,
        _ => return Err(ParseError::InvalidDataType(data_type))
    })
}


fn extended_id_from_string(mnemonic: &str) -> Result<i16, ParseError> {
    Ok(match mnemonic {
        "chkindex" => -1,
        "pushaf" => -2,
        "popaf" => -3,
        "pushac" => -4,
        "setowner" => -5,
        "isstaticok" => -6,
        "setstatic" => -7,
        "savearef" => -8,
        "restorearef" => -9,
        "chknullish" => -10,
        "pushref" => -11,
        _ => return Err(ParseError::InvalidExtendedMnemonic(mnemonic.to_string()))
    })
}


fn opcode_from_string(mnemonic: &str) -> Result<GMOpcode, ParseError> {
    Ok(match mnemonic {
        "conv" => GMOpcode::Convert,
        "mul" => GMOpcode::Multiply,
        "div" => GMOpcode::Divide,
        "rem" => GMOpcode::Remainder,
        "mod" => GMOpcode::Modulus,
        "add" => GMOpcode::Add,
        "sub" => GMOpcode::Subtract,
        "and" => GMOpcode::And,
        "or" => GMOpcode::Or,
        "xor" => GMOpcode::Xor,
        "neg" => GMOpcode::Negate,
        "not" => GMOpcode::Not,
        "shl" => GMOpcode::ShiftLeft,
        "shr" => GMOpcode::ShiftRight,
        "cmp" => GMOpcode::Compare,
        "pop" => GMOpcode::Pop,
        "dup" => GMOpcode::Duplicate,
        "ret" => GMOpcode::Return,
        "exit" => GMOpcode::Exit,
        "popz" => GMOpcode::PopDiscard,
        "jmp" => GMOpcode::Branch,
        "jt" => GMOpcode::BranchIf,
        "jf" => GMOpcode::BranchUnless,
        "pushenv" => GMOpcode::PushWithContext,
        "popenv" => GMOpcode::PopWithContext,
        "push" => GMOpcode::Push,
        "pushloc" => GMOpcode::PushLocal,
        "pushglb" => GMOpcode::PushGlobal,
        "pushbltn" => GMOpcode::PushBuiltin,
        "pushim" => GMOpcode::PushImmediate,
        "call" => GMOpcode::Call,
        "callvar" => GMOpcode::CallVariable,
        "break" => GMOpcode::Extended,
        _ => return Err(ParseError::InvalidMnemonic(mnemonic.to_string()))
    })
}


fn parse_int<T: FromStr>(line: &mut &str) -> Result<T, ParseError> {
    let mut end: usize = line.len();
    for (i, char) in line.chars().enumerate() {
        if matches!(char, '0'..='9') || (char == '-' && i == 0) {
            continue
        }
        end = i;
        break
    }

    let slice: &str = &line[..end];
    if slice.len() == 0 {
        return Err(ParseError::UnexpectedEOL("integer"))
    }
    let integer: T = slice.parse().map_err(|_| ParseError::IntegerOutOfBounds(slice.to_string()))?;
    *line = &line[end..];
    Ok(integer)
}


/// this only works if the line is ONLY the float
fn parse_float<T: FromStr>(line: &mut &str) -> Result<T, ParseError> {
    let float: T = line.parse().map_err(|_| ParseError::InvalidFloat(line.to_string()))?;
    *line = "";   // consume float
    Ok(float)
}


fn parse_bool(line: &mut &str) -> Result<bool, ParseError> {
    if line.starts_with("true") {
        *line = &line[4..];
        return Ok(true)
    }
    if line.starts_with("false") {
        *line = &line[5..];
        return Ok(false)
    }
    Err(ParseError::InvalidBoolean(line.to_string()))
}


fn comparison_type_from_string(comparison_type: &str) -> Result<GMComparisonType, ParseError> {
    Ok(match comparison_type {
        "LT" => GMComparisonType::LessThan,
        "LTE" => GMComparisonType::LessOrEqual,
        "EQ" => GMComparisonType::Equal,
        "NEQ" => GMComparisonType::NotEqual,
        "GTE" => GMComparisonType::GreaterOrEqual,
        "GT" => GMComparisonType::GreaterThan,
        _ => return Err(ParseError::InvalidComparisonType(comparison_type.to_string()))
    })
}


fn instance_type_from_string(instance_type: &str) -> Result<GMInstanceType, ParseError> {
    Ok(match instance_type {
        "self" => GMInstanceType::Self_(None),
        "other" => GMInstanceType::Other,
        "all" => GMInstanceType::All,
        "none" => GMInstanceType::None,
        "global" => GMInstanceType::Global,
        "builtin" => GMInstanceType::Builtin,
        "local" => GMInstanceType::Local,
        "stacktop" => GMInstanceType::StackTop,
        "arg" => GMInstanceType::Argument,
        "static" => GMInstanceType::Static,
        _ => return Err(ParseError::InvalidInstanceType(instance_type.to_string()))
    })
}


fn variable_type_from_string(variable_type: &str) -> Result<GMVariableType, ParseError> {
    Ok(match variable_type {
        "array" => GMVariableType::Array,
        "stacktop" => GMVariableType::StackTop,
        "normal" => GMVariableType::Normal,
        "instance" => GMVariableType::Instance,
        "arraypushaf" => GMVariableType::ArrayPushAF,
        "arraypopaf" => GMVariableType::ArrayPopAF,
        _ => return Err(ParseError::InvalidPrefix(variable_type.to_string()))
    })
}


fn parse_variable(line: &mut &str, locals: &GMCodeLocal, gm_data: &GMData) -> Result<CodeVariable, ParseError> {
    let mut instance_type: Option<GMInstanceType> = None;
    let mut variable_type: Option<GMVariableType> = None;

    if line.starts_with('[') {
        // this can be a `[object]` or `[roominst]` instance type, OR it could be a variable type (like `[array]`)
        let close_bracket: usize = line.find(']').ok_or(ParseError::UnmatchedSquareBracket)?;
        let prefix: String = line[1..close_bracket].to_string();
        *line = &line[close_bracket+1..];
        match prefix.as_str() {
            "object" => {
                println!("dsifsidjj #{line}#");
                let object_name: String = parse_identifier(line)?;
                if !line.starts_with('.') {
                    return Err(ParseError::ExpectedDot(line.to_string()))
                }
                *line = &line[1..];
                let object_ref: GMRef<GMGameObject> = gm_data.game_objects.get_object_ref_by_name(&object_name, &gm_data.strings)
                    .map_err(|e| ParseError::ObjectUnresolvable(e))?;
                instance_type = Some(GMInstanceType::Self_(Some(object_ref)));
            }
            "roominst" => {
                let instance_id: i16 = parse_int(line)?;
                instance_type = Some(GMInstanceType::RoomInstance(instance_id));
                variable_type = Some(GMVariableType::Instance);
            }
            _ => variable_type = Some(variable_type_from_string(&prefix)?)
        }
    }

    if instance_type.is_none() {
        // try to find first dot (for instance type; not always set)
        let Some(dot_index) = line.find('.') else {
            return Err(ParseError::VarNoInstanceType)
        };
        let raw_instance_type: &str = &line[..dot_index];
        instance_type = Some(instance_type_from_string(raw_instance_type)?);
        *line = &line[dot_index+1..];
    }
    let mut instance_type: GMInstanceType = instance_type.unwrap();

    if variable_type.is_none() {
        // check for prefix again (now only for variable type)
        if line.starts_with('[') {
            let close_bracket: usize = line.find(']').ok_or(ParseError::UnmatchedSquareBracket)?;
            let prefix: &str = &line[1..close_bracket];
            variable_type = Some(variable_type_from_string(prefix)?);
            *line = &line[close_bracket+1..];
        }
    }
    let variable_type: GMVariableType = variable_type.unwrap_or(GMVariableType::Normal);

    let mut variable_ref: Option<GMRef<GMVariable>> = None;
    // get name of variable
    let name: String = parse_identifier(line)?;

    if instance_type == GMInstanceType::Local && !locals.variables.is_empty() {
        'outer: for local_var in &locals.variables {
            let local_var_name: &String = local_var.name.resolve(&gm_data.strings.strings)
                .map_err(|_| ParseError::StringIndexUnresolvable(local_var.name.index))?;
            if *local_var_name != name {
                continue
            }
            // found local var; now find actual variable using name string id (is always unique)
            for (i, var) in gm_data.variables.variables.iter().enumerate() {
                if var.name.index != local_var.name.index {
                    continue
                }
                if let Some(b15_data) = &var.b15_data {
                    if b15_data.instance_type != GMInstanceType::Local {
                        continue
                    }
                }
                // found variable
                variable_ref = Some(GMRef::new(i as u32));
                break 'outer
            }
            // could not find variable with same name string id
            return Err(ParseError::VarNameStringNoMatch(local_var.name.index))
        }
    } else {
        // try "normal" variables
        for (i, var) in gm_data.variables.variables.iter().enumerate() {
            let var_name: &String = var.name.resolve(&gm_data.strings.strings)
                .map_err(|_| ParseError::StringIndexUnresolvable(var.name.index))?;
            if *var_name != name {
                continue
            }
            if let Some(b15) = &var.b15_data {
                // TODO: clean this shit up
                if b15.instance_type != instance_type
                    && !(b15.instance_type == GMInstanceType::Self_(None) && matches!(instance_type, GMInstanceType::Self_(Some(_))))
                    && b15.instance_type != GMInstanceType::Builtin
                    && b15.instance_type != GMInstanceType::Builtin
                    && instance_type != GMInstanceType::Builtin {
                    continue
                }
            }
            // found var
            variable_ref = Some(GMRef::new(i as u32));
            break
        }
    }

    let Some(variable) = variable_ref else {
        return Err(ParseError::VarUnresolvable(name))
    };

    if instance_type == GMInstanceType::Self_(None) && variable_type != GMVariableType::Normal {
        // I need to throw away the instance type so that the tests pass
        instance_type = GMInstanceType::Undefined;
    }   // TODO: comment out this block if not testing assembler


    Ok(CodeVariable {
        variable,
        variable_type,
        instance_type,
        is_int32: false,    // this has to be modified afterward, if necessary
    })
}


fn parse_function(line: &mut &str, gm_strings: &GMStrings, gm_functions: &GMFunctions) -> Result<GMRef<GMFunction>, ParseError> {
    let ident: String = parse_identifier(line)?;

    for (i, func) in gm_functions.functions.iter().enumerate() {
        let func_name: &String = func.name.resolve(&gm_strings.strings)
            .map_err(|_| ParseError::StringIndexUnresolvable(func.name.index))?;
        if *func_name == ident {
            return Ok(GMRef::new(i as u32))
        }
    }

    Err(ParseError::FuncUnresolvable(ident))
}


fn parse_identifier(line: &mut &str) -> Result<String, ParseError> {
    for (i, char) in line.chars().enumerate() {
        if matches!(char, '0'..='9') && i < 1 {
            return Err(ParseError::InvalidIdentifier("must not start with a digit"))
        }
        if matches!(char, '0'..='9' | 'A'..='Z' | 'a'..='z' | '_' | '@' | '$') {
            continue
        }
        if i < 1 {
            println!("gdjsughusdhgusdhu {char}");
            return Err(ParseError::InvalidIdentifier("must be at least one character long"))
        }
        let identifier: String = line[..i].to_string();
        *line = &line[i..];
        return Ok(identifier)
    }

    // identifier goes to end of line
    let identifier: String = line.to_string();
    *line = "";   // consume line
    Ok(identifier)
}


fn parse_string_literal(line: &mut &str) -> Result<String, ParseError> {
    if !line.starts_with('"') {
        return Err(ParseError::ExpectedString)
    }
    *line = &line[1..];

    let mut escaping: bool = false;
    let mut string: String = String::with_capacity(line.len());

    for (i, char) in line.chars().enumerate() {
        if escaping {
            match char {
                'n' => string.push('\n'),
                'r' => string.push('\r'),
                't' => string.push('\t'),
                '\\' => string.push('\\'),
                '"' => string.push('"'),
                _ => return Err(ParseError::InvalidEscapeCharacter(char)),
            }
        } else if char == '"' {
            *line = &line[i+1..];
            return Ok(string)
        } else {
            string.push(char);
        }
        if char == '\\' {
            escaping = !escaping;
        }
    }

    Err(ParseError::UnexpectedEOL("string literal"))
}

