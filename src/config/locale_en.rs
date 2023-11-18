
//           -|-
//  |   ||   /|   UEFI Variable Tool (UVT) * Module: Localization (English)
//  |   ||  / |   https://github.com/GeographicCone/UefiVarTool
//  `---'`-'  `-  Copyright © 2022 Datasone, © 2023 Piotr Szczepański

// Stores translateable user interface messages

// To translate, copy this file as locale_XX.rs.
// where XX is a two-letter ISO 639-1 language code,
// and replace the entry for locale in ../config.rs

// Application metadata defaults
pub const APP_NAME: &str = "UVT";
pub const APP_TITLE: &str = "UEFI Variable Tool";
pub const BUILD_TYPE: &str = "Manual";

// Error message prefixes and suffixes
pub const ERR_PREFIX_ARG: &str = "Argument error";
pub const ERR_PREFIX_INPUT: &str = "Input error";
pub const ERR_PREFIX_OP_GET: &str = "Get variable error";
pub const ERR_PREFIX_OP_SET: &str = "Set variable error";

// Error messages
pub const ERR_ARG: &str = "Failed to parse";
pub const ERR_ARG_ASS: [&'static str; 2] = ["Must have at most a single assignment operator", "followed by a value"];
pub const ERR_ARG_MORE: &str = "Premature end of string";
pub const ERR_ARG_NUM_DEC: &str = "Only digits 0-9 should appear in decimal value";
pub const ERR_ARG_NUM_HEX: &str = "Only digits 0-9, a-f or A-F should appear in hexadecimal value";
pub const ERR_ARG_NUM_HEX_PREFIX: &str = "Use prefix \"0x\" or \"0X\" for hexadecimal value";
pub const ERR_ARG_OPT: &str = "Unrecognized option";
pub const ERR_ARG_POS: [&'static str; 2] = ["Must have exactly one offset indicator", "followed by a value"];
pub const ERR_ARG_POS_BKT_L: &str = "Surplus opening bracket in offset identifier";
pub const ERR_ARG_POS_BKT_R: &str = "Missing closing bracket in offset identifier";
pub const ERR_ARG_SIZE_LIMIT: [&'static str; 2] = ["Number", "is too large (64 bits or 8 bytes maximum)"];
pub const ERR_ARG_SIZE_MISMATCH: [&'static str; 3] = ["Value", "too large to fit into", "bytes"];
pub const ERR_ARG_VAR_BKT_L: &str = "Surplus opening bracket in variable identifier";
pub const ERR_ARG_VAR_BKT_R: &str = "Missing closing bracket in variable identifier";
pub const ERR_INPUT: &str = "Parse error in input";
pub const ERR_INPUT_DEF: &str = "Malformed definition";
pub const ERR_INPUT_DEF_SET: [&'static str; 2] = ["Definition for", "must not specify new value to set"];
pub const ERR_INPUT_NONE: &str = "No command-line arguments or standard input: use -h or --help for usage information";
pub const ERR_INPUT_OPT: &str = "Unrecognized input option";
pub const ERR_INPUT_READ: &str = "Failed to read standard input";
pub const ERR_INPUT_REF: &str = "Malformed reference";
pub const ERR_INPUT_REF_NONE: &str = "Failed to resolve reference";
pub const ERR_INT_DEF: &str = "Internal parser error: definition retrieval attempted on wrong entry type";
pub const ERR_INT_OP: &str = "Internal parser error: operation retrieval attempted on wrong entry type";
pub const ERR_INT_SPLIT: &str = "Internal error: failed to split string into parts";
pub const ERR_UEFI_INIT: &str = "Failed to initialize UEFI services";
pub const ERR_UEFI_LOAD: &str = "Failed to initialize UEFI loaded image protocol";
pub const ERR_UEFI_LOAD_OPT: &str = "Failed to obtain UEFI image load options";
pub const ERR_UEFI_PATH_CONV: &str = "Failed to convert device image path";
pub const ERR_UEFI_PATH_FIND: &str = "Failed to locate UEFI device path protocol";
pub const ERR_UEFI_PATH_NONE: &str = "Device image path is empty";
pub const ERR_UEFI_PATH_OPEN: &str = "Failed to initialize UEFI device path protocol";
pub const ERR_UEFI_VAR_CONV: &str = "Internal error: failed to convert UEFI variable name";
pub const ERR_UEFI_VAR_GET: &str = "Failed to get variable";
pub const ERR_UEFI_VAR_GET_MANY: &str = "Use one of the above identifiers";
pub const ERR_UEFI_VAR_GET_MANY_HEAD: &str = "Which one do you mean?";
pub const ERR_UEFI_VAR_GET_MANY_ITEM: &str = " # Size: ";
pub const ERR_UEFI_VAR_GET_NONE: &str = "No such variable";
pub const ERR_UEFI_VAR_LIST: &str = "Error while enumerating UEFI variables";
pub const ERR_UEFI_VAR_SET: &str = "Failed to set variable";
pub const ERR_UEFI_VAR_SIZE: [&'static str; 3] = ["Variable size", "less than offset", "and value size"];
pub const ERR_UEFI_VAR_SIZE_GET: &str = "Failed to get variable size";

// Operations
pub const OP_SKIPPED: &str = " # Already";

// Version prompt in application header
pub const VERSION: &str = "Version";
pub const VERSION_UNKNOWN: &str = "Unknown";

// Usage information
pub const USAGE: [&'static str; 4] = ["Usage: ", "[.efi] [<Options>] <Op1> [<Op2> [... [<OpN>]]
- or - ", "[.efi] < <InputFile>
Where:
<Options>: Optional global-scope application settings
  -f --force     Force-write values even if already set as requested
  -h --help      Show usage information (precludes other operations)
  -r --restart   Upon succesful completion, perform a system restart
  -s --simulate  Do not write, only simulate actions (will still read)
<Op#>: Operation(s) to perform, can be multiple, each in the format:
  <VarName>[(<VarId>)]:<Offset>[(<Size>)][=<Value>]
Arg Overview:
  <VarName>      UEFI variable name to read or write to, case-sensitive
  <VarId>        If two variables share a name, will prompt to use this
  <Offset>       Data starting position within the given UEFI variable
  <Size>         Optional, a byte (1) by default if omitted; little-endian
  <Value>        Value to write, 8 bytes (64 bits) maximum; read if absent
  <InputFile>    Script to run, same base format as arguments + see below
File Overview:
  #                                   Comment, ignored until end of line
  !<force|restart|simulate>           Set options, same as above arguments
  <Def>,<VarName>:<Offset>[(<Size>)]  Define a variable to reference later
  @<Def>[=<Value>]                    Assign to a referenced variable
Example Command Line:
  ", " -s Lang:0x00 Lang:0x00(4)=0x01020304 Lang:0x00(4)
  Read byte at offset 0, simulate-set the dword (4 bytes), then read again
Example Input File:
  !simulate              # Simulate only, do not perform actual writes
  Language,Lang:0x00(4)  # Define a reference under the alias \"Language\"
  @Language=0x01020304   # Write to the target referred to by \"Language\"

<Offset>, <Size> and <Value> can be decimal or hexadecimal: use prefix \"0x\"
File should be a UTF-16 LE text, UEFI firmware and shell version-dependent
Output saved to a file can be re-used as input again: format is the same"];
