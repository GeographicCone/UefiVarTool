
//           -|-
//  |   ||   /|   UEFI Variable Tool (UVT) * Module: Configuration
//  |   ||  / |   https://github.com/GeographicCone/UefiVarTool
//  `---'`-'  `-  Copyright © 2022 Datasone, © 2023 Piotr Szczepański

// Stores configurable parameters together for easy adjustment

// Symbols from other modules
pub(crate) mod locale_en;
pub(crate) use locale_en as locale;  // Interface messages in English

// Application metadata
pub const APP_NAME:    Option<&str> = option_env!("CARGO_PKG_NAME");
pub const APP_TITLE:   Option<&str> = option_env!("CARGO_PKG_DESCRIPTION");
pub const APP_VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");
pub const BUILD_TYPE:  Option<&str> = option_env!("BUILD_TYPE");

// Character definitions
pub const CHAR_ARG_ASS:       char = '=';         // Argument assignment operator
pub const CHAR_ARG_BKT_L:     char = '(';         // Opening bracket for optional variable identifier or size
pub const CHAR_ARG_BKT_R:     char = ')';         // Closing bracket for optional variable identifier or size
pub const CHAR_ARG_OPT:       char = '-';         // Argument option prefix
pub const CHAR_ARG_POS:       char = ':';         // Argument offset indicator for variables
pub const CHAR_ARG_SEP:       char = ' ';         // Argument separator
pub const CHAR_BLANK_SPACE:   char = ' ';         // Space (SP) whitespace character
pub const CHAR_BLANK_TAB:     char = '\t';        // Horizontal tabulation (HT) whitespace character
pub const CHAR_FILE_EXT:      char = '.';         // File extension separator
pub const CHAR_FILE_PATH:     char = '\\';        // File path separator (single backlash, escaped)
pub const CHAR_INPUT_COMMENT: char = '#';         // Comment prefix, rest of the line is ignored
pub const CHAR_INPUT_DEF:     char = ',';         // Input definition separator
pub const CHAR_INPUT_OPT:     char = '!';         // Input option prefix
pub const CHAR_INPUT_REF:     char = '@';         // Input reference prefix
pub const CHAR_CTL_BOM:       char = '\u{FEFF}';  // Byte Order Mark (BOM) control character
pub const CHAR_CTL_CR:        char = '\r';        // Carriage Return (CR) control character
pub const CHAR_CTL_LF:        char = '\n';        // Line Feed (LF) control character

// Command-line options
pub const OPT_ARG_FORCE:         &str = "-f";
pub const OPT_ARG_FORCE_LONG:    &str = "--force";
pub const OPT_ARG_RESTART:       &str = "-r";
pub const OPT_ARG_RESTART_LONG:  &str = "--restart";
pub const OPT_ARG_SIMULATE:      &str = "-s";
pub const OPT_ARG_SIMULATE_LONG: &str = "--simulate";
pub const OPT_ARG_USAGE:         &str = "-h";
pub const OPT_ARG_USAGE_LONG:    &str = "--help";

// Input options (prefixed with CHAR_INPUT_OPT)
pub const OPT_INPUT_FORCE:    &str = "force";
pub const OPT_INPUT_RESTART:  &str = "restart";
pub const OPT_INPUT_SIMULATE: &str = "simulate";
