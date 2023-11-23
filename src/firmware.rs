
//           -|-
//  |   ||   /|   UEFI Variable Tool (UVT) * Module: Firmware
//  |   ||  / |   https://github.com/GeographicCone/UefiVarTool
//  `---'`-'  `-  Copyright © 2022 Datasone, © 2023 Piotr Szczepański

// Performs UEFI operations such as querying and setting UEFI variables

// Declare fully-qualified symbols to be used in the local scope
use alloc::{borrow::ToOwned, string::ToString, vec, vec::Vec};
use core::ptr::null_mut;
use uefi::{Char16, CStr16, CString16, Identify, Status,
    proto::{ // Protocols
        console::text::{Input, Key},
        device_path::text::{AllowShortcuts, DevicePathToText, DisplayOnly},
        loaded_image::LoadedImage},
    table::{ // Tables
        Boot, SystemTable,
        boot::{BootServices, SearchType},
        runtime::{ResetType, VariableKey}}};
use uefi_services::println;

// Symbols from other modules
use crate::config;
use crate::config::locale as msg;
use crate::data::{UefiValue, UefiVariable};
use crate::error::AppError;
use crate::string::CStr16Ext;

// Public System Functions

// Exits the application and passes the control back to the shell
pub fn exit(system_table: &SystemTable<Boot>, status: Status) {
    unsafe {

        // Call boot services to exit the application
        BootServices::exit(system_table.boot_services(),
            system_table.boot_services().image_handle(),
            status, 0, null_mut()); 

    }

}

// Retrieves the image (application executable) name
// by stripping the path and extension from the image path)
pub fn get_image_name(system_table: &SystemTable<Boot>) -> CString16 {

    // Retrieve the image path
    let image_path = get_image_path(&system_table)
        .unwrap_or_else(|_| CString16::try_from(msg::APP_NAME).unwrap());

    // Return a substring starting from the last path separator (\)
    // until the first file extension separator (.), or from the beginning
    // to the end of the string respectively, if either of these are absent
    image_path.substring(
        match image_path.find_last(config::CHAR_FILE_PATH) {
            Some(0) => 0,
            Some(i) => i + 1,
            None => 0 },
        match image_path.find_last(config::CHAR_FILE_EXT) {
            Some(0) => 0,
            Some(i) => i - 1,
            None => image_path.num_chars() - 1 })

}

// Retrieves the image path (path and name of the application executable)
pub fn get_image_path(system_table: &SystemTable<Boot>) -> Result<CString16, AppError> {

    // Store a reference to UEFI Boot Services,
    // called numerous times within this function
    let boot_services = system_table.boot_services();

    // Open the loaded-image protocol
    let loaded_image = boot_services
        .open_protocol_exclusive::<LoadedImage>(boot_services.image_handle())
        .map_err(|_| AppError::UefiLoad)?;

    // Obtain a device-path-to-text protocol handle
    let device_path_to_text_handle = *boot_services
        .locate_handle_buffer(SearchType::ByProtocol(&DevicePathToText::GUID))
        .map_err(|_| AppError::UefiPathFind)?
        .first().unwrap();

    // Open the device-path-to-text protocol
    let device_path_to_text = boot_services
        .open_protocol_exclusive::<DevicePathToText>(device_path_to_text_handle)
        .map_err(|_| AppError::UefiPathOpen)?;

    // Retrieve the image device path, error out if empty
    let image_device_path = match loaded_image.file_path() {
        Some(image_device_path) => image_device_path,
        None => return Err(AppError::UefiPathNone)
    };

    // Convert the image device path to text
    let image_device_path_text =
        device_path_to_text.convert_device_path_to_text(
            boot_services, image_device_path,
            DisplayOnly(true), AllowShortcuts(false))
        .map_err(|_| AppError::UefiPathConv)?;

    // Return the converted text as a string
    Ok(CString16::from(&*image_device_path_text))

}

// Loads the options (command-line arguments)
pub fn load_options(system_table: &SystemTable<Boot>) -> Result<Vec<CString16>, AppError> {

    // Open the loaded-image protocol
    let image = system_table.boot_services()
        .open_protocol_exclusive::<LoadedImage>(system_table.boot_services().image_handle())
        .map_err(|_| AppError::UefiLoad)?;

    // Obtain image load options
    let options = image.load_options_as_cstr16()
        .map_err(AppError::UefiLoadOpt)?;

    // Consider the possibility that the first argument (#0)
    // is not the executing image name but already a parameter

    // Note: the use case where this applies is unclear to me (@GeographicCone)
    // but since it was implemented by the original author (@datasone), he must
    // have had his reasons, so I am keeping this functionality, only making it
    // better handle different scenarios

    // Separately, if the image name is present in load options, it might include an arbitrary
    // number of quote marks ("), possibly with spaces inside, messing up argument processing
    // Example: "\EFI\Tools\uvt.efi" but also \EFI\Tools\"u v t.efi" or \"EFI"\"Tools"\uvt.efi

    // To mitigate both cases, we sanitize the loaded options, only keeping
    // those that appear vaguely legitimate, which is determined as follows:

    // All option arguments must begin with a CHAR_ARG_OPT, i.e. minus (-) sign
    // All operation arguments must contain a CHAR_ARG_POS, i.e. colon (:) sign
    // Non-conforming arguments will be silently discarded

    // Return the options, split into a CString16 vector
    Ok(options.split(config::CHAR_ARG_SEP).into_iter().filter(
        |s| s.has_first(config::CHAR_ARG_OPT)
        || s.has(config::CHAR_ARG_POS)).collect())

}

// Reads data from an input stream
// (currently used for standard input)
pub fn read_stream(input: &mut Input) -> CString16 {

    // Define ignored chars
    let is_ignored = |c|

        // Byte Order Mark (BOM)
        c == Char16::try_from(config::CHAR_CTL_BOM).unwrap()

        // Carriage Return (CR)
        || c == Char16::try_from(config::CHAR_CTL_CR).unwrap();

    // Set up the string for returned data
    let mut data = CString16::new();

    // Keep reading, while ignoring
    // special scancodes or BOM and CR
    while let Some(key) = input.read_key().expect(msg::ERR_INPUT_READ) {
        if let Key::Printable(c) = key {

            // If not ignored
            if !is_ignored(c) { 

                // Add to data
                data.push(c)

            }

        }

    }

    // Return
    data

}

// Restarts the system
pub fn restart_system(system_table: &SystemTable<Boot>) -> () {
    system_table.runtime_services()
        .reset(ResetType::WARM, Status::SUCCESS, None)
}

// Public Variable Functions

// Queries a UEFI variable at a given offset and size,
// returns the value and the operation error status
pub fn get_value(system_table: &SystemTable<Boot>,
    var_name: &CStr16, var_id: Option<usize>,
    offset: usize, length: usize) -> Result<UefiValue, AppError> {

    // Attempt to retrieve the specified variable
    let var = get_variable(&system_table, var_name, var_id)?;

    // If the specified variable is too short
    // to hold data at given offset and length
    if offset + length > var.content.len() {

        // Return an error
        return Err(
            AppError::UefiVarSize(
                (offset, length), var.content.len()));

    }

    // Retrieve the given slice of the variable
    let slice = &var.content[offset .. offset + length];

    // Return the result
    Ok(UefiValue(slice.to_vec()))

}

// Modifies a UEFI variable at a given offset and size,
// returns a flag whether changes were made, and error status
pub fn set_value(system_table: &SystemTable<Boot>,
    var_name: &CStr16, var_id: Option<usize>,
    offset: usize, length: usize, value: &UefiValue,
    force: bool, simulate: bool) -> Result<bool, AppError> {

    // Attempt to retrieve the specified variable
    let mut var = get_variable(&system_table, var_name, var_id)?;

    // If the specified variable is too short
    // to hold data at given offset and length
    if offset + length > var.content.len() {

        // Return an error
        return Err(AppError::UefiVarSize(
            (offset, length), var.content.len()));

    }

    // Retrieve the given slice of the variable
    let slice = &mut var.content[offset .. offset + length];

    // If the value is already as requested
    // and we are not being forced to write
    if !force && slice == value.0 {

        // Return with
        // no changes made
        Ok(false)

    // Otherwise
    } else {

        // Copy the new value into the slice
        slice.copy_from_slice(&value.0);

        // Unless simulating
        if !simulate {

            // Attempt to set the variable, handling a possible error
            system_table.runtime_services()
                .set_variable(&var.name, &var.vendor, var.attributes, &var.content)
                .map_err(|e| AppError::UefiVarSet(var.name.to_string(), e.status()))?;

        }

        // Return with
        // changes made
        Ok(true)

    }

}

// Private Functions

// A wrapper for UEFI Runtime Services'
// variable retrieval function, used internally
fn get_variable(system_table: &SystemTable<Boot>,
    var_name: &CStr16, var_id: Option<usize>)
    -> Result<UefiVariable, AppError> {

    // Store a reference to UEFI Runtime Services,
    // which are called numerous times within this function
    let runtime_services = system_table.runtime_services();

    // Retrieve variable name list from UEFI Runtime Services
    let keys = runtime_services.variable_keys()
        .map_err(|e| AppError::UefiVarList(e.status()))?;

    // Filter the retrieved list for matching variable names
    let mut keys = keys
        .into_iter()
        .filter(|k| {
            if let Ok(name) = k.name() {
                name == var_name
            } else {
                false
            }
        }).collect::<Vec<_>>();

    // Sort the filtered list by vendor
    keys.sort_by_key(|k| k.vendor.0);

    // If no matches were found, report an error
    if keys.is_empty() {
        return Err(AppError::UefiVarGetNone(var_name.to_string()));
    }

    // If name matched more than once, and no identifier,
    // output an identifier list and report an error
    if keys.len() > 1 && var_id.is_none() {
        get_variable_ambiguous(&system_table, keys)?;
        return Err(AppError::UefiVarGetMany);
    }

    // Pick the correct variable depending on the arguments
    let var_key =
        if keys.len() == 1 {
            // If only a single variable was found with the name,
            // it is obviously the first entry on the list
            &keys[0]
        } else {
            // If multiple variables were found, use the identifier
            &keys[var_id.unwrap()]
        };

    // Set the variable name from the name associated
    // with the key, or return a rare conversion error
    let var_name = var_key.name()?;

    // Retrieve the size of the variable, allowing for an error
    let size = runtime_services.get_variable_size(var_name, &var_key.vendor)
        .map_err(|e| AppError::UefiVarSizeGet(var_name.to_string(), e.status()))?;

    // Allocate a buffer the size of the variable
    let mut buffer = vec![0; size];

    // Retrieve the variable into the buffer using UEFI Runtime Services
    let (_, var_attr) = runtime_services.get_variable(var_name, &var_key.vendor, &mut buffer)
        .map_err(|e| AppError::UefiVarGet(var_name.to_string(), e.status()))?;

    // Return the variable
    Ok(UefiVariable {
        name: var_name.to_owned(),
        vendor: var_key.vendor,
        attributes: var_attr,
        content: buffer,
    })

}

// Handles the case where UEFI variable cannot be identified by its name,
// asks the user to reattempt the operation providing a unique identifier
fn get_variable_ambiguous(system_table: &SystemTable<Boot>, keys: Vec<VariableKey>)
    -> Result<(), AppError> {

    println!("{}", msg::ERR_UEFI_VAR_GET_MANY_HEAD);

    // Iterate through variables with the matching name
    for(i, key) in keys.into_iter().enumerate() {

        // Retrieve metadata for each variable
        let id = i;
        let name = key.name()?;
        let size = system_table.runtime_services()
            .get_variable_size(name, &key.vendor)
            .map_err(|e| AppError::UefiVarSizeGet(name.to_string(), e.status()))?;

        // Output the resulting information
        println!("{}({:#04x}){}{:#06x}", name, id, msg::ERR_UEFI_VAR_GET_MANY_ITEM, size);

    }

    // Return
    Ok(())

}
