
//           -|-
//  |   ||   /|   UEFI Variable Tool (UVT) * Module: Error
//  |   ||  / |   https://github.com/GeographicCone/UefiVarTool
//  `---'`-'  `-  Copyright © 2022 Datasone, © 2023 Piotr Szczepański

// Allows for error handling in a single centralized manner

// Declare fully-qualified symbols
// to be used in the local scope
use alloc::string::String;
use core::fmt::{Display, Formatter, Result as FmtResult};
use uefi::{data_types::FromSliceWithNulError, proto::loaded_image::LoadOptionsError, Status};

// Symbols from other modules
use crate::config;
use crate::config::locale as msg;

// Error list
// Note: some errors handled directly,
// listed for completeness only
#[derive(Debug)]
pub enum AppError {

    // Args
    Arg(String),
    ArgAss,
    ArgMore(String),
    ArgNone,
    ArgNumDec(String),
    ArgNumHex(String),
    ArgNumHexPrefix(String),
    ArgOpt,
    ArgPos,
    ArgPosBktL,
    ArgPosBktR,
    ArgSizeLimit(String),
    ArgSizeMismatch(usize, usize),
    ArgVarBktL,
    ArgVarBktR,

    // Input
    Input(String),
    InputDef(String),
    InputDefSet(String),
    InputNone,
    InputOpt(String),
    //InputRead,           // firmware::read_stream()
    InputRef(String),
    InputRefNone(String),

    // Internal
    //IntDef,              // data::InputEntry::as_def()
    //IntOp,               // data::InputEntry::as_op()
    //IntSplit,            // string::CStr16Ext::split()

    // UEFI
    //UefiInit,            // main::main()
    UefiLoad,
    UefiLoadOpt(LoadOptionsError),
    UefiPathConv,
    UefiPathFind,
    UefiPathNone,
    UefiPathOpen,
    UefiVarConv(FromSliceWithNulError),
    UefiVarGet(String, Status),
    UefiVarGetMany,
    UefiVarGetNone(String),
    UefiVarList(Status),
    UefiVarSet(String, Status),
    UefiVarSize((usize, usize), usize),
    UefiVarSizeGet(String, Status),

}

// Error display implementation
impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {

            // Argument

            // Parse error (argument)
            Self::Arg(string) => {
                write!(f, "{}: {string}", msg::ERR_ARG)
            }

            // Surplus assignment
            Self::ArgAss => {
                write!(f, "{} ({}) {}", msg::ERR_ARG_ASS[0], config::CHAR_ARG_ASS, msg::ERR_ARG_POS[1])
            }

            // More expected
            Self::ArgMore(string) => {
                write!(f, "{}: {string}", msg::ERR_ARG_MORE)
            }

            // Empty argument list (not actual error)
            Self::ArgNone => {
                write!(f, "")
            }

            // Decimal number format
            Self::ArgNumDec(string) => {
                write!(f, "{} {string}", msg::ERR_ARG_NUM_DEC)
            }

            // Hexadecimal number format
            Self::ArgNumHex(string) => {
                write!(f, "{} {string}", msg::ERR_ARG_NUM_HEX)
            }

            // Hexadecimal number prefix
            Self::ArgNumHexPrefix(string) => {
                write!(f, "{} {string}", msg::ERR_ARG_NUM_HEX_PREFIX)
            }

            // Offset left bracket
            Self::ArgPosBktL => {
                write!(f, "{}", msg::ERR_ARG_POS_BKT_L)
            }

            // Offset right bracket
            Self::ArgPosBktR => {
                write!(f, "{}", msg::ERR_ARG_POS_BKT_R)
            }

            // Unrecognized option
            Self::ArgOpt => {
                write!(f, "{}", msg::ERR_ARG_OPT)
            }

            // Incorrect offset specification
            Self::ArgPos => {
                write!(f, "{} ({}) {}", msg::ERR_ARG_POS[0], config::CHAR_ARG_POS, msg::ERR_ARG_POS[1])
            }

            // Variable identifier left bracket
            Self::ArgVarBktL => {
                write!(f, "{}", msg::ERR_ARG_VAR_BKT_L)
            }

            // Variable identifier right bracket
            Self::ArgVarBktR => {
                write!(f, "{}", msg::ERR_ARG_VAR_BKT_R)
            }

            // Number too large
            Self::ArgSizeLimit(string) => {
                write!(f, "{} {string} {}",
                    msg::ERR_ARG_SIZE_LIMIT[0], msg::ERR_ARG_SIZE_LIMIT[1])
            }

            // New value larger than size specified
            Self::ArgSizeMismatch(value, size) => {
                write!(f, "{} {value:#0width$x} {} {size} {}",
                    msg::ERR_ARG_SIZE_MISMATCH[0], msg::ERR_ARG_SIZE_MISMATCH[1],
                    msg::ERR_ARG_SIZE_MISMATCH[2], width = 2 + size * 2)
            }

            // Input

            // Parse error (input)
            Self::Input(string) => {
                write!(f, "{}: {string}", msg::ERR_INPUT)
            }

            // Input definition malformed
            Self::InputDef(string) => {
                write!(f, "{} \"{string}\"", msg::ERR_INPUT_DEF)
            }

            // Definition attempts a set operation
            Self::InputDefSet(string) => {
                write!(f, "{} \"{string}\" {}",
                    msg::ERR_INPUT_DEF_SET[0], msg::ERR_INPUT_DEF_SET[1])
            }

            // No input or command-line arguments
            Self::InputNone => {
                write!(f, "{}", msg::ERR_INPUT_NONE)
            }

            // Input option unrecognized
            Self::InputOpt(string) => {
                write!(f, "{} \"{string}\"", msg::ERR_INPUT_OPT)
            }

            // Input reference
            Self::InputRef(string) => {
                write!(f, "{} \"{string}\"", msg::ERR_INPUT_REF)
            }

            // Input reference not found
            Self::InputRefNone(string) => {
                write!(f, "{} \"{string}\"", msg::ERR_INPUT_REF_NONE)
            }

            // UEFI

            // Failed to initialize loaded-image protocol
            Self::UefiLoad => {
                write!(f, "{}", msg::ERR_UEFI_LOAD)
            }

            // Failed to obtain loaded image options
            Self::UefiLoadOpt(e) => {
                write!(f, "{}: {e:?}", msg::ERR_UEFI_LOAD_OPT)
            }

            // Failed to convert device image path
            Self::UefiPathConv => {
                write!(f, "{}", msg::ERR_UEFI_PATH_CONV)
            }

            // Failed to locate device-path protocol
            Self::UefiPathFind => {
                write!(f, "{}", msg::ERR_UEFI_PATH_FIND)
            }

            // Device image path empty
            Self::UefiPathNone => {
                write!(f, "{}", msg::ERR_UEFI_PATH_NONE)
            }

            // Failed to initialize device-path protocol
            Self::UefiPathOpen => {
                write!(f, "{}", msg::ERR_UEFI_PATH_OPEN)
            }

            // Failed to convert variable name
            Self::UefiVarConv(details) => {
                write!(f, "{}: {details:?}", msg::ERR_UEFI_VAR_CONV)
            }

            // Failed to get variable
            Self::UefiVarGet(name, status) => {
                write!(f, "{}: \"{name}\" ({status:?})", msg::ERR_UEFI_VAR_GET)
            }

            // Ambiguous variable reference
            Self::UefiVarGetMany => {
                write!(f, "{}", msg::ERR_UEFI_VAR_GET_MANY)
            }

            // No such variable
            Self::UefiVarGetNone(name) => {
                write!(f, "{}: \"{name}\"", msg::ERR_UEFI_VAR_GET_NONE)
            }

            // Failed to list variables
            Self::UefiVarList(status) => {
                write!(f, "{} ({status:?})", msg::ERR_UEFI_VAR_LIST)
            }

            // Failed to set variable
            Self::UefiVarSet(name, status) => {
                write!(f, "{}: \"{name}\" ({status:?})", msg::ERR_UEFI_VAR_SET)
            }

            // Size too small given offset and length
            Self::UefiVarSize((offset, length), size) => {
                write!(f, "{} {size:#06x} {} {offset:#06x} {} {length}",
                    msg::ERR_UEFI_VAR_SIZE[0], msg::ERR_UEFI_VAR_SIZE[1], msg::ERR_UEFI_VAR_SIZE[2])
            }

            // Failed to get variable size
            Self::UefiVarSizeGet(name, status) => {
                write!(f, "{}: \"{name}\" ({status:?})", msg::ERR_UEFI_VAR_SIZE_GET)
            }

        }

    }

}

// Implementation of a UEFI-specific conversion error
impl From<FromSliceWithNulError> for AppError {

    fn from(value: FromSliceWithNulError) -> Self {

        // Use default error handling
        Self::UefiVarConv(value)

    }

}
