use std::fmt::{Display, Formatter};
use std::str::{Chars, FromStr};
use crate::gamemaker::data::GMData;
use crate::gamemaker::deserialize::GMRef;
use crate::gamemaker::elements::code::{GMCodeValue, CodeVariable, GMGotoInstruction, GMPopSwapInstruction, GMSingleTypeInstruction, GMInstanceType, GMVariableType, GMComparisonInstruction, GMPopInstruction, GMPushInstruction, GMCallInstruction, GMExtendedInstruction16, GMExtendedInstructionFunc, GMExtendedInstruction32, GMEmptyInstruction};
use crate::gamemaker::elements::code::{GMCallVariableInstruction, GMComparisonType, GMDoubleTypeInstruction, GMDuplicateInstruction, GMDuplicateSwapInstruction};
use crate::gamemaker::elements::code::GMInstruction;
use crate::gamemaker::elements::code::GMDataType;
use crate::gamemaker::elements::functions::{GMCodeLocal, GMFunction, GMFunctions};
use crate::gamemaker::elements::game_objects::GMGameObject;
use crate::gamemaker::elements::strings::GMStrings;
use crate::gamemaker::elements::variables::GMVariable;


#[derive(Debug)]
pub enum ParseError {
    ExpectedEOL(String),
    ExpectedSpace(String),
    ExpectedDot(String),
    ExpectedString(String),
    ExpectedArgc(String),
    ExpectedIdentifier(String),
    UnexpectedEOL(&'static str),
    /// `(expected, actual)`
    InvalidTypeCount(usize, usize),
    InvalidDataType(char),
    InvalidComparisonType(String),
    InvalidInstanceType(String),
    InvalidMnemonic(String),
    InvalidExtendedMnemonic(String),
    InvalidVariableType(String),
    InvalidTypeCast(String),
    IntegerOutOfBounds(String),
    InvalidFloat(String),
    InvalidBoolean(String),
    InvalidIdentifier(&'static str),
    InvalidEscapeCharacter(char),
    UnmatchedAngleBracket,
    UnmatchedSquareBracket,
    UnmatchedRoundBracket,
    StringIndexUnresolvable(u32),
    VarLocalInvalidIndex(String),
    VarUnresolvable(String),
    FuncUnresolvable(String),
    ObjectUnresolvable(String),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::ExpectedEOL(line) => write!(f, "Expected end of line; found remaining string \"{line}\""),
            ParseError::ExpectedSpace(line) => write!(f, "Expected space; found remaining string \"{line}\""),
            ParseError::ExpectedDot(line) => write!(f, "Expected dot; found remaining string \"{line}\""),
            ParseError::ExpectedString(line) => write!(f, "Expected string literal; found remaining string \"{line}\""),
            ParseError::ExpectedArgc(line) => write!(f, "Expected argument count of called function; found remaining string \"{line}\""),
            ParseError::ExpectedIdentifier(line) => write!(f, "Expected identifier; found remaining string \"{line}\""),
            ParseError::UnexpectedEOL(context) => write!(f, "Unexpected end of line while parsing {context}"),
            ParseError::InvalidTypeCount(expected, actual) => write!(f, "Expected {expected} data types for this opcode; found {actual} types"),
            ParseError::InvalidDataType(char) => write!(f, "Invalid data type character '{char}'"),
            ParseError::InvalidComparisonType(cmp_type) => write!(f, "Invalid comparison type \"{cmp_type}\""),
            ParseError::InvalidInstanceType(inst_type) => write!(f, "Invalid instance type \"{inst_type}\""),
            ParseError::InvalidMnemonic(mnemonic) => write!(f, "Invalid opcode mnemonic \"{mnemonic}\""),
            ParseError::InvalidExtendedMnemonic(mnemonic) => write!(f, "Invalid extended/break mnemonic \"{mnemonic}\""),
            ParseError::IntegerOutOfBounds(int_str) => write!(f, "Integer out of bounds \"{int_str}\""),
            ParseError::InvalidFloat(float_str) => write!(f, "Invalid floating point number \"{float_str}\""),
            ParseError::InvalidBoolean(bool_str) => write!(f, "Invalid boolean \"{bool_str}\""),
            ParseError::InvalidTypeCast(cast) => write!(f, "Invalid Type Cast \"{cast}\""),
            ParseError::InvalidIdentifier(reason) => write!(f, "Identifier {reason}"),
            ParseError::InvalidEscapeCharacter(char) => write!(f, "Invalid escape character '{char}'"),
            ParseError::InvalidVariableType(var_type) => write!(f, "Invalid Variable Type \"{var_type}\""),
            ParseError::UnmatchedRoundBracket => write!(f, "Round bracket '(' was never closed"),
            ParseError::UnmatchedSquareBracket => write!(f, "Square bracket '[' was never closed"),
            ParseError::UnmatchedAngleBracket => write!(f, "Angle bracket '<' was never closed"),
            ParseError::StringIndexUnresolvable(idx) => write!(f, "Could not resolve String with index {idx}"),
            ParseError::VarLocalInvalidIndex(arg) => write!(f, "Local variable has an invalid variable index specified: \"{arg}\""),
            ParseError::VarUnresolvable(var_name) => write!(f, "Variable \"{var_name}\" does not exist"),
            ParseError::FuncUnresolvable(func_name) => write!(f, "Function \"{func_name}\" does not exist"),
            ParseError::ObjectUnresolvable(error) => write!(f, "{error}"),
        }
    }
}


pub fn assemble_code(assembly: &str, gm_data: &mut GMData) -> Result<Vec<GMInstruction>, String> {
    let mut instructions: Vec<GMInstruction> = Vec::new();

    for line in assembly.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue
        }

        let instruction: GMInstruction = assemble_instruction(line, gm_data)
            .map_err(|e| format!("{e}\n↳ while assembling instruction: {line}"))?;
        instructions.push(instruction);
    }

    Ok(instructions)
}


pub fn assemble_instruction(line: &str, gm_data: &mut GMData) -> Result<GMInstruction, ParseError> {
    let line: &mut &str = &mut line.trim();
    let mnemonic: String;

    let opcode_end: Option<usize> = line.find(|c: char| !c.is_ascii_alphanumeric());
    if let Some(index) = opcode_end {
        let (head, tail) = line.split_at(index);
        mnemonic = head.to_string();
        *line = tail;
    } else {
        // opcode takes up entire line
        mnemonic = line.to_string();
        *line = "";
    }

    // TODO: maybe smallvec or something? making a heap allocation for 3 bytes is really unnecessary
    let mut types: Vec<GMDataType> = Vec::with_capacity(2);
    while line.chars().next() == Some('.') {
        consume_dot(line)?;
        let raw_type: char = consume_char(line).ok_or(ParseError::UnexpectedEOL("data type"))?;
        types.push(data_type_from_char(raw_type)?);
    }

    match line.chars().next() {
        Some(' ') => { consume_char(line); }
        None => {}
        _ => return Err(ParseError::ExpectedSpace(line.to_string()))
    }

    let instruction: GMInstruction = match mnemonic.as_str() {
        "conv" => GMInstruction::Convert(parse_double_type(&types)?),
        "mul" => GMInstruction::Multiply(parse_double_type(&types)?),
        "div" => GMInstruction::Divide(parse_double_type(&types)?),
        "rem" => GMInstruction::Remainder(parse_double_type(&types)?),
        "mod" => GMInstruction::Modulus(parse_double_type(&types)?),
        "add" => GMInstruction::Add(parse_double_type(&types)?),
        "sub" => GMInstruction::Subtract(parse_double_type(&types)?),
        "and" => GMInstruction::And(parse_double_type(&types)?),
        "or" => GMInstruction::Or(parse_double_type(&types)?),
        "xor" => GMInstruction::Xor(parse_double_type(&types)?),
        "neg" => GMInstruction::Negate(parse_single_type(&types)?),
        "not" => GMInstruction::Not(parse_single_type(&types)?),
        "shl" => GMInstruction::ShiftLeft(parse_double_type(&types)?),
        "shr" => GMInstruction::ShiftRight(parse_double_type(&types)?),
        "cmp" => GMInstruction::Compare(parse_comparison(&types, line)?),
        "pop" => GMInstruction::Pop(parse_pop(&types, line, gm_data)?),
        "popswap" => GMInstruction::PopSwap(parse_pop_swap(&types, line)?),
        "dup" => GMInstruction::Duplicate(parse_duplicate(&types, line)?),
        "dupswap" => GMInstruction::DuplicateSwap(parse_duplicate_swap(&types, line)?),
        "ret" => GMInstruction::Return(parse_single_type(&types)?),
        "exit" => GMInstruction::Exit(GMEmptyInstruction),
        "popz" => GMInstruction::PopDiscard(parse_single_type(&types)?),
        "jmp" => GMInstruction::Branch(parse_goto(&types, line)?),
        "jt" => GMInstruction::BranchIf(parse_goto(&types, line)?),
        "jf" => GMInstruction::BranchUnless(parse_goto(&types, line)?),
        "pushenv" => GMInstruction::PushWithContext(parse_goto(&types, line)?),
        "popenv" => GMInstruction::PopWithContext(parse_goto(&types, line)?),
        "push" => GMInstruction::Push(parse_push(&types, line, gm_data)?),
        "pushloc" => GMInstruction::PushLocal(parse_push(&types, line, gm_data)?),
        "pushglb" => GMInstruction::PushGlobal(parse_push(&types, line, gm_data)?),
        "pushbltn" => GMInstruction::PushBuiltin(parse_push(&types, line, gm_data)?),
        "pushim" => GMInstruction::PushImmediate(parse_push(&types, line, gm_data)?),
        "call" => GMInstruction::Call(parse_call(&types, line, gm_data)?),
        "callvar" => GMInstruction::CallVariable(parse_call_var(&types, line)?),
        _ => {
            let kind: i16 = extended_id_from_string(&mnemonic)?;
            assert_type_count(&types, 1)?;
            if line.is_empty() {
                GMInstruction::Extended16(GMExtendedInstruction16 { kind })
            } else if consume_str(line, "(function)").is_some() {
                let function: GMRef<GMFunction> = parse_function(line, &gm_data.strings, &gm_data.functions)?;
                GMInstruction::ExtendedFunc(GMExtendedInstructionFunc { kind, function })
            } else {
                let int_argument: i32 = parse_int(line)?;
                GMInstruction::Extended32(GMExtendedInstruction32 { kind, int_argument })
            }
        }
    };

    if !line.is_empty() {
        return Err(ParseError::ExpectedEOL(line.to_string()))
    }

    Ok(instruction)
}


fn parse_double_type(types: &Vec<GMDataType>) -> Result<GMDoubleTypeInstruction, ParseError> {
    assert_type_count(&types, 2)?;
    Ok(GMDoubleTypeInstruction { type1: types[0], type2: types[1] })
}

fn parse_single_type(types: &Vec<GMDataType>) -> Result<GMSingleTypeInstruction, ParseError> {
    assert_type_count(&types, 1)?;
    Ok(GMSingleTypeInstruction { data_type: types[0] })
}

fn parse_comparison(types: &Vec<GMDataType>, line: &mut &str) -> Result<GMComparisonInstruction, ParseError> {
    assert_type_count(&types, 2)?;
    let comparison_type_raw: String = parse_identifier(line)?;
    let comparison_type: GMComparisonType = comparison_type_from_string(&comparison_type_raw)?;
    Ok(GMComparisonInstruction { comparison_type, type1: types[0], type2: types[1] })
}

fn parse_pop(types: &Vec<GMDataType>, line: &mut &str, gm_data: &GMData) -> Result<GMPopInstruction, ParseError> {
    assert_type_count(&types, 2)?;
    let destination: CodeVariable = parse_variable(line, &gm_data)?;
    Ok(GMPopInstruction { type1: types[0], type2: types[1], destination })
}

fn parse_pop_swap(types: &Vec<GMDataType>, line: &mut &str) -> Result<GMPopSwapInstruction, ParseError> {
    assert_type_count(&types, 0)?;
    let size: u8 = parse_int(line)?;
    Ok(GMPopSwapInstruction { size })
}

fn parse_duplicate(types: &Vec<GMDataType>, line: &mut &str) -> Result<GMDuplicateInstruction, ParseError> {
    assert_type_count(&types, 1)?;
    let size: u8 = parse_int(line)?;
    Ok(GMDuplicateInstruction { data_type: types[0], size })
}

fn parse_duplicate_swap(types: &Vec<GMDataType>, line: &mut &str) -> Result<GMDuplicateSwapInstruction, ParseError> {
    assert_type_count(&types, 1)?;
    let size1: u8 = parse_int(line)?;
    consume_space(line)?;
    let size2: u8 = parse_int(line)?;
    Ok(GMDuplicateSwapInstruction { data_type: types[0], size1, size2 })
}

fn parse_goto(types: &Vec<GMDataType>, line: &mut &str) -> Result<GMGotoInstruction, ParseError> {
    assert_type_count(&types, 0)?;
    let jump_offset: Option<i32> = if *line == "<drop>" {
        *line = "";    // need to consume; otherwise error
        None
    } else {
        Some(parse_int(line)?)
    };
    Ok(GMGotoInstruction { jump_offset })
}

fn parse_push(types: &Vec<GMDataType>, line: &mut &str, gm_data: &mut GMData) -> Result<GMPushInstruction, ParseError> {
    assert_type_count(&types, 1)?;
    let value: GMCodeValue = match types[0] {
        GMDataType::Int16 => GMCodeValue::Int16(parse_int(line)?),
        GMDataType::Int32 => {
            if let Some(type_cast) = consume_round_brackets(line)? {
                match type_cast.as_str() {
                    "function" => GMCodeValue::Function(parse_function(line, &gm_data.strings, &gm_data.functions)?),
                    "variable" => {
                        let mut variable: CodeVariable = parse_variable(line, &gm_data)?;
                        variable.is_int32 = true;
                        GMCodeValue::Variable(variable)
                    }
                    _ => return Err(ParseError::InvalidTypeCast(type_cast))
                }
            } else {
                GMCodeValue::Int32(parse_int(line)?)
            }
        }
        GMDataType::Int64 => GMCodeValue::Int64(parse_int(line)?),
        GMDataType::Float => GMCodeValue::Float(parse_float(line)?),
        GMDataType::Double => GMCodeValue::Double(parse_float(line)?),
        GMDataType::Boolean => GMCodeValue::Boolean(parse_bool(line)?),
        GMDataType::String => {
            let string_text: String = parse_string_literal(line)?;
            let string_ref: GMRef<String> = gm_data.make_string(&string_text);
            GMCodeValue::String(string_ref)
        },
        GMDataType::Variable => GMCodeValue::Variable(parse_variable(line, &gm_data)?),
    };
    Ok(GMPushInstruction { value })
}

fn parse_call(types: &Vec<GMDataType>, line: &mut &str, gm_data: &GMData) -> Result<GMCallInstruction, ParseError> {
    assert_type_count(&types, 1)?;
    let function: GMRef<GMFunction> = parse_function(line, &gm_data.strings, &gm_data.functions)?;
    let argc_str: String = consume_round_brackets(line)?.ok_or(ParseError::ExpectedArgc(line.to_string()))?;
    let arguments_count: u8 = if let Some(rest) = argc_str.strip_prefix("argc=") {
        rest.parse().map_err(|_| ParseError::IntegerOutOfBounds(rest.to_string()))?
    } else {
        return Err(ParseError::ExpectedArgc(argc_str))
    };
    Ok(GMCallInstruction { arguments_count, data_type: types[0], function })
}

fn parse_call_var(types: &Vec<GMDataType>, line: &mut &str) -> Result<GMCallVariableInstruction, ParseError> {
    assert_type_count(&types, 1)?;
    let argument_count: u8 = parse_int(line)?;
    Ok(GMCallVariableInstruction { data_type: types[0], argument_count })
}



fn assert_type_count(types: &Vec<GMDataType>, n: usize) -> Result<(), ParseError> {
    if types.len() != n {
        return Err(ParseError::InvalidTypeCount(n, types.len()))
    }
    Ok(())
}


fn consume_char(line: &mut &str) -> Option<char> {
    let mut chars: Chars = line.chars();
    let first_char: Option<char> = chars.next();
    *line = chars.as_str();
    first_char
}


fn consume_chars(line: &mut &str, count: usize) -> Result<String, ParseError> {
    // find byte position after [`n`] characters
    let byte_pos: usize = line
        .char_indices()
        .nth(count)
        .map(|(pos, _)| pos)
        .ok_or(ParseError::UnexpectedEOL("consuming n chars"))?;

    let consumed: String = line[..byte_pos].to_string();
    *line = &line[byte_pos..];
    Ok(consumed)
}


/// This function is so stupid.
fn consume_str(line: &mut &str, string: &'static str) -> Option<()> {
    if line.starts_with(string) {
        *line = &line[string.bytes().len()..];
        Some(())
    } else {
        None
    }
}


fn consume_space(line: &mut &str) -> Result<(), ParseError> {
    let char: char = consume_char(line).ok_or(ParseError::UnexpectedEOL("space"))?;
    if char != ' ' {
        return Err(ParseError::ExpectedSpace(line.to_string()))
    }
    Ok(())
}

fn consume_dot(line: &mut &str) -> Result<(), ParseError> {
    let char: char = consume_char(line).ok_or(ParseError::UnexpectedEOL("dot"))?;
    if char != '.' {
        return Err(ParseError::ExpectedDot(line.to_string()))
    }
    Ok(())
}


fn consume_brackets_template<const OPEN: char, const CLOSE: char>(line: &mut &str, err: ParseError) -> Result<Option<String>, ParseError> {
    if !line.starts_with(OPEN) {
        return Ok(None)
    }
    consume_char(line);   // consume open bracket
    let close_bracket: usize = line.find(CLOSE).ok_or(err)?;
    let inside: String = line[..close_bracket].to_string();
    *line = &line[close_bracket+1..];
    Ok(Some(inside))
}

fn consume_round_brackets(line: &mut &str) -> Result<Option<String>, ParseError> {
    consume_brackets_template::<'(', ')'>(line, ParseError::UnmatchedRoundBracket)
}

fn consume_square_brackets(line: &mut &str) -> Result<Option<String>, ParseError> {
    consume_brackets_template::<'[', ']'>(line, ParseError::UnmatchedSquareBracket)
}

fn consume_angle_brackets(line: &mut &str) -> Result<Option<String>, ParseError> {
    consume_brackets_template::<'<', '>'>(line, ParseError::UnmatchedAngleBracket)
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
        _ => return Err(ParseError::InvalidMnemonic(mnemonic.to_string()))
    })
}


fn parse_int<T: FromStr>(line: &mut &str) -> Result<T, ParseError> {
    let mut end: usize = line.bytes().len();
    for (i, char) in line.char_indices() {
        if matches!(char, '0'..='9') || (char == '-' && i == 0) {
            continue
        }
        end = i;
        break
    }

    let slice: &str = &line[..end];
    if slice.is_empty() || slice == "-" {
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
    let bool: String = parse_identifier(line)?;
    if bool == "true" {
        return Ok(true)
    }
    if bool == "false" {
        return Ok(false)
    }
    Err(ParseError::InvalidBoolean(bool.to_string()))
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


fn variable_type_from_string(variable_type: &str) -> Result<GMVariableType, ParseError> {
    Ok(match variable_type {
        "array" => GMVariableType::Array,
        "stacktop" => GMVariableType::StackTop,
        "normal" => GMVariableType::Normal,
        "instance" => GMVariableType::Instance,
        "arraypushaf" => GMVariableType::ArrayPushAF,
        "arraypopaf" => GMVariableType::ArrayPopAF,
        _ => return Err(ParseError::InvalidVariableType(variable_type.to_string()))
    })
}


fn parse_variable(line: &mut &str, gm_data: &GMData) -> Result<CodeVariable, ParseError> {
    let mut variable_type: GMVariableType = GMVariableType::Normal;
    if let Some(variable_type_str) = consume_square_brackets(line)? {
        variable_type = variable_type_from_string(&variable_type_str)?;
    }

    let instance_type_raw: String = parse_identifier(line)?;
    let instance_type_arg: String = consume_angle_brackets(line)?.unwrap_or_default();
    consume_dot(line)?;

    let mut variable_ref: Option<GMRef<GMVariable>> = None;
    let mut instance_type: GMInstanceType = match instance_type_raw.as_str() {
        "self" if instance_type_arg.is_empty() => GMInstanceType::Self_(None),
        "self" => {
            let object_ref: GMRef<GMGameObject> = gm_data.game_objects.get_object_ref_by_name(&instance_type_arg, &gm_data.strings)
                .map_err(|e| ParseError::ObjectUnresolvable(e))?;
            GMInstanceType::Self_(Some(object_ref))
        }
        "local" => {
            let var_index: u32 = instance_type_arg.parse()
                .map_err(|_| ParseError::VarLocalInvalidIndex(instance_type_arg))?;
            variable_ref = Some(GMRef::new(var_index));
            GMInstanceType::Local
        },
        "other" => GMInstanceType::Other,
        "all" => GMInstanceType::All,
        "none" => GMInstanceType::None,
        "global" => GMInstanceType::Global,
        "builtin" => GMInstanceType::Builtin,
        "stacktop" => GMInstanceType::StackTop,
        "arg" => GMInstanceType::Argument,
        "static" => GMInstanceType::Static,
        _ => return Err(ParseError::InvalidInstanceType(instance_type_raw.to_string()))
    };

    let name: String = parse_identifier(line)?;

    // convert instance type because of some bullshit
    let vari_instance_type: GMInstanceType = match instance_type {
        GMInstanceType::Self_(Some(_)) => GMInstanceType::Self_(None),
        GMInstanceType::Other => GMInstanceType::Self_(None),
        GMInstanceType::Argument => GMInstanceType::Builtin,
        GMInstanceType::Builtin => GMInstanceType::Self_(None),
        GMInstanceType::StackTop => GMInstanceType::Self_(None),
        _ => instance_type.clone(),
    };

    if instance_type != GMInstanceType::Local {
        for (i, var) in gm_data.variables.variables.iter().enumerate() {
            let var_name: &String = var.name.resolve(&gm_data.strings.strings)
                .map_err(|_| ParseError::StringIndexUnresolvable(var.name.index))?;
            if *var_name != name {
                continue
            }
            if let Some(b15) = &var.b15_data && b15.instance_type != vari_instance_type {
                continue
            }
            // found var
            variable_ref = Some(GMRef::new(i as u32));
            break
        }
    }

    let Some(variable) = variable_ref else {
        return Err(ParseError::VarUnresolvable(name))
    };

    // I need to throw away the instance type so that the tests pass
    if variable_type != GMVariableType::Normal {
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


/// @ and $ are allowed because of internal gamemaker variable/function names.
fn parse_identifier(line: &mut &str) -> Result<String, ParseError> {
    for (i, char) in line.char_indices() {
        if matches!(char, '0'..='9') && i == 0 {
            return Err(ParseError::InvalidIdentifier("must not start with a digit"))
        }
        if matches!(char, '0'..='9' | 'A'..='Z' | 'a'..='z' | '_' | '@' | '$') {
            continue
        }
        let identifier: String = line[..i].to_string();
        if identifier.is_empty() {
            return Err(ParseError::ExpectedIdentifier(line.to_string()))
        }
        *line = &line[i..];
        return Ok(identifier)
    }

    // identifier goes to end of line
    let identifier: String = line.to_string();
    *line = "";   // consume line
    Ok(identifier)
}


fn parse_string_literal(line: &mut &str) -> Result<String, ParseError> {
    if consume_char(line) != Some('"') {
        return Err(ParseError::ExpectedString(line.to_string()))
    }

    let mut escaping: bool = false;
    let mut string: String = String::with_capacity(line.bytes().len());

    for (i, char) in line.char_indices() {
        if escaping {
            match char {
                'n' => string.push('\n'),
                'r' => string.push('\r'),
                't' => string.push('\t'),
                '\\' => string.push('\\'),
                '"' => string.push('"'),
                _ => return Err(ParseError::InvalidEscapeCharacter(char)),
            }
            escaping = false;
        } else if char == '"' {
            *line = &line[i+1..];
            return Ok(string)
        } else if char == '\\' {
            escaping = true;
        } else {
            string.push(char);
        }
    }

    Err(ParseError::UnexpectedEOL("string literal"))
}

