
//           -|-
//  |   ||   /|   UEFI Variable Tool (UVT) * Module: Data
//  |   ||  / |   https://github.com/GeographicCone/UefiVarTool
//  `---'`-'  `-  Copyright © 2022 Datasone, © 2023 Piotr Szczepański

// Defines data types and structures used throughout the application

// Declare fully-qualified symbols
// to be used in the local scope
use alloc::{borrow::ToOwned, format, string::String, vec::Vec};
use core::fmt::{Display, Formatter, Result as FmtResult};
use uefi::{CString16, table::runtime::{VariableAttributes, VariableVendor}};

// Symbols from other modules
use crate::config;
use crate::config::locale as msg;
use crate::error::AppError;

// Operation target
// Structure identifying a value in a UEFI variable

#[derive(Clone, Debug, Default)]
pub struct OperationTarget {

    pub id: Option<usize>,  // Optional to tell namesakes
    pub name: CString16,    // Name of the UEFI variable
    pub offset: usize,      // Offset within the variable
    pub size: usize,        // Value data length from offset

}

// Operation type
// Whether the value is being retrieved or written

#[derive(Clone, Copy, Debug, Default)]
pub enum OperationType {

    #[default]
    Get,         // Query the current value
    Set(usize),  // Assign a different value

}

// Operation argument
// Structure identifying the action to take,
// as well as the target of an operation

#[derive(Clone, Debug, Default)]
pub struct ArgOperation {
    pub action: OperationType,    // Get or set
    pub target: OperationTarget,  // Variable, offset, length
}

// Implementation
impl ArgOperation {

    // Operation argument validation, with error handling
    pub fn validate(&self) -> Result<(), AppError> {

        // For an assignment operation
        if let OperationType::Set(value) = self.action {

            // Make sure that the value to be assigned
            // fits within the specified size of the value
            if value > (1 << (self.target.size * 8)) {

                // Return an error if that's not the case
                Err(AppError::ArgSizeMismatch(value, self.target.size))

            // Fits
            } else {

                // Pass
                Ok(())

            }

        // For a query
        } else {

            // Pass
            Ok(())

        }

    }

    // Retrieval as a string, together with the current value
    pub fn to_string_with_val(&self, value: &UefiValue) -> String {

        // Retrieve the variable name
        let name = &self.target.name;

        // Retrieve the identifier, if present
        let id_string = match self.target.id {
            None => "".to_owned(),
            Some(id) => format!("{}{id}{}", config::CHAR_ARG_BKT_L, config::CHAR_ARG_BKT_R),
        };

        // Retrieve the offset
        let offset = self.target.offset;

        // If value size is more than a single byte
        let size_string = if self.target.size == 1 {
            "".to_owned()
        } else {

            // Retrieve the optional size (if not a byte)
            format!("{}{}{}", config::CHAR_ARG_BKT_L, self.target.size, config::CHAR_ARG_BKT_R)

        };

        // Retrieve the current value, using UefiValue's implementation
        let value_string = value.to_string_with_size(self.target.size);

        // Format the resulting information and return
        format!("{name}{id_string}{}{offset:#06x}{size_string}{}{value_string}",
            config::CHAR_ARG_POS, config::CHAR_ARG_ASS)

    }

}

// Option argument list
#[derive(Debug)]
pub enum ArgOption {

    Force,     // Force-write identical values
    Restart,   // Restart system when done
    Simulate,  // Simulate, do not write
    Usage,     // Show usage information

}

// Every argument is either
// an operation, or an option
#[derive(Debug)]
pub enum Arg {

    Operation(ArgOperation),  // Get or set a given UEFI value
    Option(ArgOption),        // Set state in application scope

}

// Argument structure
// holds all arguments
#[derive(Debug, Default)]
pub struct Args {

    // Operation arguments
    pub op: Vec<ArgOperation>,

    // Option arguments
    // Application-scope state
    pub force: bool,
    pub restart: bool,
    pub simulate: bool,
    pub usage: bool,

}

// Implementation of
// argument validation
impl Args {

    pub fn validate(&self) -> Result<(), AppError> {

        // If asked to show usage information
        if self.usage {

            // Do not validate, since
            // nothing else will be done
            Ok(())

        } else {

            // Iterate through the operations,
            // attempting to validate each of them
            self.op.iter().try_for_each(|i| i.validate())

        }

    }

}

// Input entry types
#[derive(Debug)]
pub enum InputEntry {

    //ArgOperation(ArgOperation),
    //ArgOption(ArgOption),
    Operation(ArgOperation),  // Get or set a given UEFI value
    Option(ArgOption),        // Set state in application scope

    // Define a target to be referenced elsewhere in the file
    TargetDefinition { name: CString16, target: OperationTarget },

    // Reference of a target defined elsewhere in the file
    TargetReference { name: CString16, action: OperationType },
}

// Implementation
// for input entries
impl InputEntry {

    // Retrieval of the target definition for the operation
    pub fn as_def(&self) -> (&CString16, &OperationTarget) {

        match self {
            Self::TargetDefinition { name, target } => (name, target),

            // Fail if attempted on an incorrect entry type
            _ => panic!("{}", msg::ERR_INT_DEF)
        }

    }

    // Retrieval of an argument for the operation
    pub fn as_op(&self) -> &ArgOperation {

        match self {
            Self::Operation(op) => op,

            // Fail if attempted on an incorrect entry type
            _ => panic!("{}", msg::ERR_INT_OP)

        }

    }

}

// UEFI Value
// Byte array at a given offset within a UEFI variable

pub struct UefiValue(pub Vec<u8>);

// Implementation
impl UefiValue {

    // Assignment from a given value and length
    pub fn from_usize(value: usize, length: usize) -> Self {
        let value = value & ((1 << (length * 8)) - 1);
        Self(value.to_le_bytes()[0 .. length].to_vec())
    }

    // Retrieval as a string of a given length
    pub fn to_string_with_size(&self, length: usize) -> String {
        let mut bytes = [0; 8];
        bytes[0 .. self.0.len()].copy_from_slice(&self.0);

        format!("{:#0size$x}", usize::from_ne_bytes(bytes), size = 2 + length * 2)

    }
}

// Implementation: formatting for display
// (used by to_string(), currently never called)
impl Display for UefiValue {

    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut bytes = [0; 8];
        bytes[0 .. self.0.len()].copy_from_slice(&self.0);

        write!(f, "{:#04x}", usize::from_ne_bytes(bytes))

    }

}

// UEFI Variable
// A configuration-data storage unit implemented by UEFI
// Each variable can store numerous configuration settings

pub struct UefiVariable {
    pub attributes: VariableAttributes,
    pub content: Vec<u8>,
    pub name: CString16,
    pub vendor: VariableVendor,
}
