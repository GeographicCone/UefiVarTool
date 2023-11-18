
//           -|-
//  |   ||   /|   UEFI Variable Tool (UVT) * Base Application
//  |   ||  / |   https://github.com/GeographicCone/UefiVarTool
//  `---'`-'  `-  Copyright © 2022 Datasone, © 2023 Piotr Szczepański

// Provides the entry point and launches all operations

#![no_main]  // No default main entry point
#![no_std]   // No underlying standard library

// Need to explicitly import alloc crate
// to use allocation in a no_std context
extern crate alloc;

// Modules
mod config;    // Stores configurable parameters together for easy adjustment
mod data;      // Defines data types and structures used throughout the application
mod error;     // Allows for error handling in a single centralized manner
mod firmware;  // Performs UEFI operations such as querying and setting UEFI variables
mod parse;     // Processes command-line and stream (standard) input into data structures
mod string;    // Provides string manipulation routines, including an extension to CStr16

// Declare fully-qualified symbols
// to be used in the local scope
use uefi::prelude::*;
use uefi_services::println;

// Symbols from other modules
use config::locale as msg;
use data::Args;
use error::AppError;
use firmware::{exit, get_image_name, get_value, load_options, read_stream, restart_system, set_value};
use parse::{parse_args, parse_input};

#[entry] // Main entry point to the application
fn main(_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {

    // Initialize UEFI services
    uefi_services::init(&mut system_table)
        .expect(msg::ERR_UEFI_INIT);

    // Print name and version header, including
    // firmware vendor and version, and UEFI revision
    println!("# {} ({}) {} {}-{} @ {} {}.{:02} UEFI {}", 
        config::APP_TITLE.unwrap_or_else(|| msg::APP_TITLE), config::APP_NAME.unwrap_or_else(|| msg::APP_NAME),
        msg::VERSION, config::APP_VERSION.unwrap_or_else(|| msg::VERSION_UNKNOWN), 
        config::BUILD_TYPE.unwrap_or_else(|| msg::BUILD_TYPE), system_table.firmware_vendor(),
        system_table.firmware_revision() >> 16, system_table.firmware_revision() & 0xFFFFu32,
        system_table.uefi_revision());

    // Set the default exit status
    let mut status = Status::SUCCESS;

    // Attempt to load and parse command-line arguments,
    // determine subsequent actions based on the outcome
    let args = match parse_args(load_options(&system_table).unwrap()) {

        // Parse success
        // Continue with arguments
        Ok(args) => args,

        // Empty argument list
        Err(AppError::ArgNone) => {

            // Parse the data from standard input
            match parse_input(read_stream(system_table.stdin())) {

                // If success, use
                // data as arguments
                Ok(args) => args,

                // Nothing to do
                Err(AppError::InputNone) => {

                    // Show message
                    println!("{}", AppError::InputNone);
                    return Status::SUCCESS;

                }

                // Failure
                Err(e) => {

                    // Show input file parser error
                    println!("{}: {e}", msg::ERR_PREFIX_INPUT);

                    // Exit after showing usage information
                    status = Status::INVALID_PARAMETER;
                    Args { usage: true, ..Default::default() }

                }

            }

        }

        // Failure
        Err(e) => {

            // Show argument parser error
            println!("{}: {e}", msg::ERR_PREFIX_ARG);

            // Exit after showing usage information
            status = Status::INVALID_PARAMETER;
            Args { usage: true, ..Default::default() }

        }

    };

    // If show usage
    if args.usage {

        // Only output usage and exit
        show_usage(&system_table, status);

    }

    // Iterate through operations
    for op in args.op {

        // Process each operation and retain its status
        let status = process_op(&system_table,
            &op, args.force, args.simulate);

        // If an operation failed
        if status != Status::SUCCESS {

            // Interrupt
            return status;

        }

    }

    // If restart requested
    if args.restart {

        // Restart the system upon succesful completion
        restart_system(&system_table);

    }

    // Return no error
    Status::SUCCESS

}

// Process an argument operation, returning its status
fn process_op(system_table: &SystemTable<Boot>,
    op: &data::ArgOperation, force: bool, simulate: bool) -> Status {

    // Variable name, offset and size
    let name = &op.target.name;
    let size = op.target.size;
    let offset = op.target.offset;

    // Operation type
    match op.action {

        // Get current value
        data::OperationType::Get => {

            // Perform retrieval
            match get_value(&system_table,
                name, op.target.id, offset, size) {

                // Success
                Ok(value) =>

                    // Output the formatted value
                    println!("{}", op.to_string_with_val(&value)),

                // Failure
                Err(e) => {

                    // Show error message and interrupt processing
                    println!("{}: {e}", msg::ERR_PREFIX_OP_GET);
                    return Status::ABORTED;

                }

            }

        }

        // Set new value
        data::OperationType::Set(value) => {

            // Initialize the new value
            let value = data::UefiValue::from_usize(value, size);

            // Perform the assignment
            match set_value(&system_table,
                name, op.target.id, offset, size, &value, force, simulate) {

                // Success
                Ok(written) => {

                    // Output the formatted value,
                    // adding a comment if no writing occurred
                    println!("{}{}", op.to_string_with_val(&value),
                        if let false = written {
                            msg::OP_SKIPPED
                        } else {
                            ""
                        });

                }

                // Failure
                Err(e) => {

                    // Show error message and interrupt processing
                    println!("{}: {e}", msg::ERR_PREFIX_OP_SET);
                    return Status::ABORTED;

                }

            }

        }

    }

    // Return no error
    Status::SUCCESS

}

// Shows the usage information and exits the application
fn show_usage(system_table: &SystemTable<Boot>, status: Status) {

    // Obtain the application image name
    let image_name = get_image_name(&system_table);

    // Output the usage information, substituting the image name
    println!("{}{image_name}{}{image_name}{}{image_name}{}",
        msg::USAGE[0], msg::USAGE[1], msg::USAGE[2], msg::USAGE[3]);

    // Make a call to UEFI boot services to exit
    exit(&system_table, status);

}
