
//           -|-
//  |   ||   /|   UEFI Variable Tool (UVT) * Module: Parse
//  |   ||  / |   https://github.com/GeographicCone/UefiVarTool
//  `---'`-'  `-  Copyright © 2022 Datasone, © 2023 Piotr Szczepański

// Processes command-line and stream input into data structures

// Declare fully-qualified symbols to be used in the local scope
use alloc::{borrow::{Cow, ToOwned}, format, string::ToString, vec::Vec};
use uefi::{CStr16, CString16, data_types::EqStrUntilNul};

// Symbols from other modules
use crate::config;
use crate::data::{
    Arg, Args, ArgOperation, ArgOption,
    InputEntry, OperationTarget, OperationType};
use crate::error::AppError;
use crate::parse_multiple;
use crate::string::{CStr16Ext, try_next_char};

// Command-Line Arguments

// Parses the command-line arguments received from load options
pub fn parse_args(args: Vec<CString16>) -> Result<Args, AppError> {

    // Skip if nothing to parse
    if args.len() == 0 {
        return Err(AppError::ArgNone);
    }

    // Iterate through arguments, trying to parse each
    // with either the operation or the option parser
    let args = args.into_iter().map(|s| {
        parse_multiple!(&s, parse_arg_option, parse_arg_operation)

            // Interrupt on error
            .map_err(|e| AppError::Arg(format!("\"{s}\" - {e}")))

        // Collect into a common Arg vector
        }).collect::<Result<Vec<_>, _>>()?;

    // Collect operations
    let operations = args.iter().filter(|arg| matches!(arg, Arg::Operation(_)))
        .map(|arg| {
            if let Arg::Operation(arg) = arg { arg.clone() } else { unreachable!() }
        }).collect::<Vec<_>>();

    // Collect options
    let options = args.iter().filter(|arg| matches!(arg, Arg::Option(_)))
        .map(|arg| {
            if let Arg::Option(arg) = arg { arg } else { unreachable!() }
        }).collect::<Vec<_>>();

    // Initialize the argument structure
    let mut args = Args::default();

    // Assign the options
    for option in options {
        match option {

            // Force-write even if already set
            ArgOption::Force => args.force = true,

            // Restart system on completion
            ArgOption::Restart => args.restart = true,

            // Simulate, do not write
            ArgOption::Simulate => args.simulate = true,

            // Show usage information
            ArgOption::Usage => args.usage = true,

        }

    }

    // Assign the operations
    args.op = operations;

    // Validate arguments
    // Also handles errors
    args.validate()?;

    // Ready
    Ok(args)

}

// Attempts to parse a command-line operation as a general argument
fn parse_arg_operation(arg: &CStr16) -> Result<Arg, AppError> {

    // This function is just a wrapper that returns
    // a general instead of an operation argument
    Ok(Arg::Operation(parse_operation(arg)?))

}

// Attempts to parse a command-line option as a general argument
fn parse_arg_option(key: &CStr16) -> Result<Arg, AppError> {

    // Force-write even if already set
    if key.eq_str_until_nul(config::OPT_ARG_FORCE)
        || key.eq_str_until_nul(config::OPT_ARG_FORCE_LONG) {

        Ok(Arg::Option(ArgOption::Force))

    // Restart system on completion
    } else if key.eq_str_until_nul(config::OPT_ARG_RESTART)
        || key.eq_str_until_nul(config::OPT_ARG_RESTART_LONG) {

        Ok(Arg::Option(ArgOption::Restart))

    // Simulate, do not write
    } else if key.eq_str_until_nul(config::OPT_ARG_SIMULATE)
        || key.eq_str_until_nul(config::OPT_ARG_SIMULATE_LONG) {

        Ok(Arg::Option(ArgOption::Simulate))

    // Show usage information
    } else if key.eq_str_until_nul(config::OPT_ARG_USAGE)
        || key.eq_str_until_nul(config::OPT_ARG_USAGE_LONG) {

        Ok(Arg::Option(ArgOption::Usage))

    // Unknown
    } else {

        // Return an error
        Err(AppError::ArgOpt)

        // Note: currently obstructed
        // due to parser limitations

    }

}

// Input Stream (Standard Input)

// Parses an input script read from a stream such as standard input
pub fn parse_input(input: CString16) -> Result<Args, AppError> {

    // Error if nothing to parse
    if input.is_empty() {
        return Err(AppError::InputNone);
    }

    // Split input into lines
    let lines = input.split(config::CHAR_CTL_LF);

    // Remove comments and whitespace from each line
    let lines = lines.into_iter().filter_map(|s| {

        // Set up a helper variable
        let mut line_filtered = s.clone();

        // If the line has a comment
        if s.has(config::CHAR_INPUT_COMMENT) {

            // Find out where the comment begins
            let comment_start = s.find_first(config::CHAR_INPUT_COMMENT).unwrap();
	
            // If at first character,
            // ignore the entire line
            if comment_start == 0 {
                return None;
            }
	
            // Filter out the comment portion
            line_filtered = s.substring(0, comment_start - 1);
	
        }

        // Trim leading and trailing whitespace
        line_filtered = line_filtered.trim();

        // Skip line if completely empty
        if line_filtered.is_empty() {
            return None;
        }

        // Return the result
        Some(line_filtered)

    });

    // Parse the input into the entries
    let entries = lines.map(|s| {
        parse_multiple!(&s,
            parse_input_option, parse_target_def,
            parse_input_operation, parse_target_ref) })
        .collect::<Result<Vec<_>, _>>()?;

    // Collect parsed target definitions
    let target_defs = entries.iter().filter(
        |e| matches!(e, InputEntry::TargetDefinition { .. }))
        .map(|e| e.as_def()).collect::<Vec<_>>();

    // Collect parsed input operations
    let mut operations = entries.iter().filter(
        |e| matches!(e, InputEntry::Operation(_))).map(
            |e| e.as_op()).cloned().collect::<Vec<_>>();

    // Collect parsed target references
    let target_refs = entries.iter().filter(
        |e| matches!(e, InputEntry::TargetReference { .. }));

    // Collect parsed input operations with references
    let mut operations_ref = target_refs.map(|e| {
        match e {
            InputEntry::TargetReference { name, action } => {

                // Find the target definition for the reference
                let (_, target) = target_defs.iter().find(
                    |(def_name, _)| name == *def_name).ok_or_else(
                        || AppError::InputRefNone(name.to_string()))?;
                Ok(ArgOperation {
                    target: (*target).clone(), action: *action })
            }
            _ => unreachable!() }})
        .collect::<Result<Vec<_>, AppError>>()?;

    // Include the operations with references
    // together with those defined directly
    operations.append(&mut operations_ref);

    // Set option arguments from input
    let force = entries.iter().any(     // Force write
        |e| matches!(e, InputEntry::Option(ArgOption::Force)));

    let restart = entries.iter().any(   // Restart when done
        |e| matches!(e, InputEntry::Option(ArgOption::Restart)));

    let simulate = entries.iter().any(  // Simulate, do not write
        |e| matches!(e, InputEntry::Option(ArgOption::Simulate)));

    // Return the complete argument structure
    Ok(Args { op: operations, force, restart, simulate, usage: false })

}

// Attempts to parse a command-line operation as an input entry
fn parse_input_operation(arg: &CStr16) -> Result<InputEntry, AppError> {

    // This function is just a wrapper that returns
    // an input entry instead of an operation argument
    Ok(InputEntry::Operation(crate::parse::parse_operation(arg)?))

}

// Attempts to parse a command-line option as an input entry
fn parse_input_option(arg: &CStr16) -> Result<InputEntry, AppError> {

    // Remove the option prefix
    let named_arg = arg.strip_first(config::CHAR_INPUT_OPT)
        .ok_or_else(|| AppError::InputOpt(arg.to_string()))?;

    // Parse the option
    if named_arg.eq_str_until_nul(config::OPT_INPUT_FORCE) {

        // Force-write even if already set
        Ok(InputEntry::Option(ArgOption::Force))

    } else if named_arg.eq_str_until_nul(config::OPT_INPUT_RESTART) {

        // Restart system on completion
        Ok(InputEntry::Option(ArgOption::Restart))

    } else if named_arg.eq_str_until_nul(config::OPT_INPUT_SIMULATE) {

        // Simulate, do not write
        Ok(InputEntry::Option(ArgOption::Simulate))

    } else {

        // Unrecognized option error
        Err(AppError::InputOpt(arg.to_string()))

    }

}

// Common (Command-Line & Input Stream)

// Attempts to parse a command-line argument as an operation argument
fn parse_operation(arg: &CStr16) -> Result<ArgOperation, AppError> {

    // Split the argument at the offset specification
    let mut arg_split = arg.split(config::CHAR_ARG_POS);

    // Every argument must have
    // exactly one offset indicator
    if arg_split.len() != 2 {
        Err(AppError::ArgPos)?
    }

    // Determine the variable name
    // Note: swap_remove() is O(1), remove is O(n)
    let mut name = arg_split.swap_remove(0);

    // Determine the variable identifier
    // Empty by default, can be defined in brackets
    let mut id = None;
    if name.has(config::CHAR_ARG_BKT_L) {

        // Split the variable name at the opening bracket
        let mut arg_split = name.split(config::CHAR_ARG_BKT_L);

        // Part left of offset may have
        // at most a single bracket
        if arg_split.len() != 2 {
            Err(AppError::ArgVarBktL)?
        }

        // Remove the matching closing bracket
        // Error out if no closing bracket present
        let id_string = arg_split[1]
            .strip_last(config::CHAR_ARG_BKT_R)
            .ok_or(AppError::ArgVarBktR)?;

        // Parse the variable identifier as
        // either a decimal or a hexadecimal number
        id = Some(parse_multiple!(&id_string, parse_value_dec, parse_value_hex)?);

        // Update the variable name
        // to remove the part in brackets
        name = arg_split.swap_remove(0);

    }

    // Determine the offset (position within variable)
    let offset = arg_split.swap_remove(0);

    // Determine the operation type and the new data
    // to be assigned if operation type is to set the value
    let (mut offset, op_type) = parse_operation_type(&offset)?;

    // Determine the value size
    // Defaults to a byte (1), can be defined in brackets
    let mut size = 1;
    if offset.has(config::CHAR_ARG_BKT_L) {

        // Split the offset at the opening bracket
        let mut arg_split = offset.split(config::CHAR_ARG_BKT_L);

        // Part right of offset may have
        // at most a single bracket
        if arg_split.len() != 2 {
            Err(AppError::ArgPosBktL)?
        }

        // Remove the matching closing bracket
        // Error out if no closing bracket present
        let size_string = arg_split[1]
            .strip_last(config::CHAR_ARG_BKT_R)
            .ok_or(AppError::ArgPosBktR)?;

        // Parse the size as either a decimal or a hexadecimal number
        size = parse_multiple!(&size_string, parse_value_dec, parse_value_hex)?;

        // Update the offset to remove the part in brackets
        offset = Cow::Owned(arg_split.swap_remove(0));

    }

    // Parse the offset value as either a decimal or a hexadecimal number
    let offset = parse_multiple!(&offset, parse_value_hex, parse_value_dec)?;

    // Return the populated data structure
    Ok(ArgOperation { action: op_type,
        target: OperationTarget { id, name, offset, size }})

}

// Attempts to parse operation type,
// and optionally the new data to be set
fn parse_operation_type(arg: &CStr16)
    -> Result<(Cow<CStr16>, OperationType), AppError> {

    // If operation is an assignment
    if arg.has(config::CHAR_ARG_ASS) {

        // Split the string at the opening bracket
        let mut arg_split = arg.split(config::CHAR_ARG_ASS);

        // There can only be
        // a single assignment operator
        if arg_split.len() != 2 {
            Err(AppError::ArgAss)?
        }

        // Parse the value to be assigned as either a decimal or a hexadecimal number
        let value = parse_multiple!(&arg_split[1], parse_value_hex, parse_value_dec)?;

        // Set the operation type to assignment (set) and the new
        // value to be assigned, update the offset to only the part
        // on the left-hand size of the assignment operator
        Ok((Cow::Owned(arg_split.swap_remove(0)), OperationType::Set(value)))

    } else {

        // Set the operation type to retrieval (get)
        // Return the offset argument as received
        Ok((Cow::Borrowed(arg), OperationType::Get))

    }

}

// Attempts to parse a decimal value
fn parse_value_dec(value: &CStr16) -> Result<usize, AppError> {

    // Split the input string into a character vector
    let chars = value.iter().map(|&c| char::from(c)).collect::<Vec<_>>();

    // Only ASCII digits are allowed in a decimal value
    if chars.iter().any(|c| !c.is_ascii_digit()) {
        Err(AppError::ArgNumDec(format!("\"{}\"", value.to_string())))?
    }

    // Iterate through the characters:
    // Take the ASCII value of each (48-57), and deduct the value of 0 (48)
    // Multiply the accumulator by 10 at each step and add to the total
    let value = chars.into_iter()
        .fold(0usize, |acc, n| acc * 10 + (n as u8 - b'0') as usize);

    // Return
    Ok(value)

}

// Attempts to parse a hexadecimal value
fn parse_value_hex(value: &CStr16) -> Result<usize, AppError> {

    // Define an iterator over the input string
    let mut str_iter = value.iter().map(|&c| char::from(c));

    // Retrieve the initial two characters of the string
    let c0 = try_next_char(&mut str_iter, value)?;
    let c1 = try_next_char(&mut str_iter, value)?;

    // Check the prefix
    match (c0, c1) {

        // The hexadecimal
        // prefix is present
        ('0', 'x') => {}
        ('0', 'X') => {}

        // Report an error in hexadecimal string formatting
        _ => return Err(AppError::ArgNumHexPrefix(format!("\"{}\"", value.to_string()))),

    }

    // Check if within the size constraint
    if value.num_bytes() > 2 * (18 + 1) {
        return Err(AppError::ArgSizeLimit(value.to_string()));
    }

    // Convert each byte (char) to its hexadecimal value
    let value = str_iter
        .map(|c| c.to_digit(16).map(|n| n as u8))
        .collect::<Option<Vec<u8>>>()
        .ok_or_else(|| AppError::ArgNumHex(format!("\"{}\"", value.to_string())))?;

    // Sum all the values, multiplied according their respective positions
    let length = value.len();
    let value = value.iter().enumerate().fold(0usize, |acc, (i, &n)| {
        acc + ((n as usize) << (4 * (length - i - 1)))
    });

    // Return
    Ok(value)

}

// Definitions & References
// (Input Stream Only)

// Attempts to parse a target definition
pub fn parse_target_def(arg: &CStr16) -> Result<InputEntry, AppError> {

    // Split into two at input definition sepator
    let (name, target) = arg.split_once(config::CHAR_INPUT_DEF)
        .ok_or_else(|| AppError::InputDef(arg.to_string()))?;

    // Attempt to parse the target as an operation
    let operation = parse_operation(&target)?;

    // Check operation type
    if let OperationType::Get = operation.action {

        // Add the target definition entry
        Ok(InputEntry::TargetDefinition {
            name, target: operation.target })

    } else {

        // Value setting not allowed in a definition
        Err(AppError::InputDefSet(target.to_string()))

    }

}

// Attempts to parse a target reference
fn parse_target_ref(arg: &CStr16)
    -> Result<InputEntry, AppError> {

    // Check if prefix is the first character in reference
    let arg = if arg.has_first(config::CHAR_INPUT_REF) {

        // Remove the prefix from reference
        arg.strip_first(config::CHAR_INPUT_REF)
            .ok_or_else(|| AppError::Input(arg.to_string()))?

    } else {

        // Report an error: prefix is mandatory
        Err(AppError::InputRef(arg.to_string()))?

    };

    // Retrieval is the default,
    // name is the whole argument
    let mut action = OperationType::Get;
    let mut name = CString16::from(arg);

    // Override the settings for an assignment operation
    // if assignment operator appears in the argument
    if arg.has(config::CHAR_ARG_ASS) {

        // Split into two at argument assignment operator
        // keep the operator in the value as it is required
        let ass = arg.find_first(config::CHAR_ARG_ASS).unwrap();
        name = arg.substring(0, ass - 1);
        let value = arg.substring(ass, arg.num_chars() - 1);

        // Retrieve the operation type (and the new value if set)
        action = parse_target_ref_operation_type(&value)?;

    }

    // Return the reference
    Ok(InputEntry::TargetReference { action, name: name.to_owned() })

}

// Attempts to parse operation type,
// and optionally the new data to be set
fn parse_target_ref_operation_type(arg: &CStr16) -> Result<OperationType, AppError> {

    // This function is just a wrapper that returns
    // only the operation type and discards the offset
    Ok(parse_operation_type(arg)?.1)

}
