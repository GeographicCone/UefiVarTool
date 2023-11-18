
//           -|-
//  |   ||   /|   UEFI Variable Tool (UVT) * Module: String
//  |   ||  / |   https://github.com/GeographicCone/UefiVarTool
//  `---'`-'  `-  Copyright © 2022 Datasone, © 2023 Piotr Szczepański

// Provides string manipulation routines, including an extension to CStr16

// Declare fully-qualified symbols
// to be used in the local scope
use alloc::{string::ToString, vec::Vec};
use uefi::{Char16, CStr16, CString16, data_types::chars::NUL_16};

// Symbols from other modules
use crate::config;
use crate::config::locale as msg;
use crate::error::AppError;

// Converts a Char16 vector to CString16
pub fn char16_vec_to_cstring16(string: Vec<Char16>) -> CString16 {

    // A copy and char validity check is required
    CString16::try_from(
        string.into_iter().map(u16::from).collect::<Vec<_>>()).unwrap()

    // Note: CString16 does not use repr(transparent), which means it cannot
    // be constructed directly, even with unsafe functions, hence the workaround

}

// Tries to retrieve the next char of a string
pub fn try_next_char(
    iter: &mut impl Iterator<Item = char>,
    string: &CStr16) -> Result<char, AppError> {

    // Error if no next char
    iter.next().ok_or_else(|| AppError::ArgMore(string.to_string()))

}

// A recursive variadic macro definition
// to apply an arbitrary number of parsers
#[macro_export]
macro_rules! parse_multiple {

    // Base case with just a single parser
    ($input:expr, $parser:expr) => {{

        // Call the parser
        // to process the input
        $parser($input)

    }};

    // Variadic case with an arbitrary number of parsers
    ($input:expr, $parser:expr, $($parsers:expr),* $(,)? ) => {{

        // Invoke the next parser and check status
        if let Ok(val) = $parser($input) {

            // Match
            Ok(val)

        } else {

            // Continue with the remaining parsers
            parse_multiple!($input, $($parsers),*)

        }

    }};

}

// Trait (interface) for an extension to CStr16
// which is the UEFI-specific equivalent to str
pub trait CStr16Ext {

    // Finds the first location of the given char
    fn find_first(&self, search: char) -> Option<usize>;

    // Finds the last location of the given char
    fn find_last(&self, search: char) -> Option<usize>;

    // Checks for the presence of the given char
    fn has(&self, search: char) -> bool;

    // Checks if the string starts with the given char
    fn has_first(&self, search: char) -> bool;

    // Splits the string into parts separated by the given char
    fn split(&self, search: char) -> Vec<CString16>;

    // Splits the string into two parts separated by the given char
    fn split_once(&self, search: char) -> Option<(CString16, CString16)>;

    // Removes the specified leading char from the string
    fn strip_first(&self, search: char) -> Option<&CStr16>;

    // Removes the specified trailing char from the string
    fn strip_last(&self, search: char) -> Option<CString16>;

    // Returns a substring given start and end indices
    fn substring(&self, start: usize, end: usize) -> CString16;

    // Removes whitespace on both ends
    fn trim(&self) -> CString16;

}

// Implementation on top of CStr16
impl CStr16Ext for CStr16 {

    // Finds the first location of the given char
    fn find_first(&self, search: char) -> Option<usize> {

        // Convert the search character to match data type
        let search = Char16::try_from(search).unwrap();

        // Find the location of the given char, error if none
        let index = self.as_slice().iter().position(|&c| c == search)?;

        // Return
        Some(index)

    }

    // Finds the last location of the given char
    fn find_last(&self, search: char) -> Option<usize> {

        // Convert the search character to match data type
        let search = Char16::try_from(search).unwrap();

        // Find the location of the given char, error if none
        let index = self.as_slice().iter().rposition(|&c| c == search)?;

        // Return
        Some(index)

    }

    // Checks for the presence of the given char
    fn has(&self, search: char) -> bool {

        // Convert the search character to match data type
        let search = Char16::try_from(search).unwrap();

        // Iterate through the chars
        // return true if any matches
        self.iter().any(|&c| c == search)

    }

    // Checks if the string starts with the given char
    fn has_first(&self, search: char) -> bool {

        // Also returns false for empty strings
        *self.iter().next().unwrap_or(&NUL_16) == search.try_into().unwrap()

    }

    // Splits the string into parts separated by the given char
    fn split(&self, search: char) -> Vec<CString16> {

        // Convert the search character to match data type
        let search = Char16::try_from(search).unwrap();

        // Set up output and helper variables
        let mut split_strings = Vec::new();
        let mut current_string = Vec::new();

        // Iterate through input string
        for c in self.iter() {

            // If not a split char
            if *c != search {

                // Append it to the current string
                current_string.push(u16::from(*c))

            // Split
            } else {

                // Terminate the current string
                current_string.push(0u16);

                // Append the current string to the array
                split_strings.push(current_string.try_into()
                    .expect(msg::ERR_INT_SPLIT));

                // Reset the current string
                current_string = Vec::new()

            }

        }

        // Deal with the leftover portion
        if !current_string.is_empty() {

            // Terminate the leftover
            current_string.push(0u16);

            // Append the leftover string to the array
            split_strings.push(current_string.try_into()
                .expect(msg::ERR_INT_SPLIT));

        }

        // Return the result
        split_strings

    }

    // Splits the string into two parts at the first
    // occurrence of the given character, trims both parts
    fn split_once(&self, search: char) -> Option<(CString16, CString16)> {

        // Convert the search character to match data type
        let search = Char16::try_from(search).unwrap();

        // Find the location of the given char, error if none
        let index = self.iter().position(|&c| c == search)?;

        // Save a slice on each side of the split as a vector
        let mut former = self.as_slice()[.. index].to_vec();
        let mut latter = self.as_slice()[index + 1 ..].to_vec();

        // Terminate both
        former.push(NUL_16);
        latter.push(NUL_16);

        // Convert each slice to a CString16
        let former = char16_vec_to_cstring16(former);
        let latter = char16_vec_to_cstring16(latter);

        // Return the result, trimmed
        Some((former.trim(), latter.trim()))

    }

    // Removes the specified leading char from the string
    fn strip_first(&self, search: char) -> Option<&CStr16> {

        // Check if the char is indeed at the beginning
        if *self.iter().next()? == Char16::try_from(search).unwrap() {

            // Create a reference
            let reference = unsafe {

                // Point to a substring bypassing the first char
                CStr16::from_u16_with_nul_unchecked(
                    &*(&self.as_slice_with_nul()[1 .. ] as *const [Char16] as *const [u16]))

            };

            // Return the reference
            Some(reference)

        } else {

            // Return nothing
            None

        }

    }

    // Removes the specified trailing char from the string
    fn strip_last(&self, search: char) -> Option<CString16> {

        // Set up a helper variable
        let str = self.to_u16_slice();

        // Check if the char is indeed the suffix
        if *str.last()? == u16::from(Char16::try_from(search).unwrap()) {

            // Create a mutable buffer
            let mut buffer = self
                .as_slice()  // Without the terminating NUL_16
                .iter()
                .map(|&c| u16::from(c))
                .collect::<Vec<_>>();

            // Change the last element to zero
            *buffer.last_mut()? = 0;

            // Return the substring
            Some(CString16::try_from(buffer).unwrap())

        } else {

            // Return nothing
            None

        }

    }

    // Returns a substring given start and end indices
    fn substring(&self, start: usize, end: usize) -> CString16 {

        // Take a slice without the whitespace characters
        let mut result = self.as_slice()[start ..= end].to_vec();

        // Terminate the slice
        result.push(NUL_16);

        // Convert character vector to string
        char16_vec_to_cstring16(result)

    }

    // Removes whitespace on both ends
    fn trim(&self) -> CString16 {

        // Do not trim empty strings
        if self.is_empty() {
            return CString16::new();
        }

        // Define a way for determining whitespace chars
        let is_whitespace = |&c|
            c == Char16::try_from(config::CHAR_BLANK_SPACE).unwrap()    // Space
            || c == Char16::try_from(config::CHAR_BLANK_TAB).unwrap();  // Tab

        // Locate the first and last
        // non-whitespace characters
        let length = self.num_chars() - 1;
        let mut start = length;
        let mut end = 0;

        // Find the first non-whitespace character
        for (i, c) in self.iter().enumerate() {
            if !is_whitespace(c) {
                start = i;
                break;
            }
        }

        // Find the last non-whitespace character
        for (i, c) in self.as_slice().iter().rev().enumerate() {
            if !is_whitespace(c) {
                end = length - i;
                break;
            }
        }

        // Result
        if start > end {
            // Empty string
            CString16::new()
        } else {
            // Actual substring
            self.substring(start, end)
        }

    }

}
