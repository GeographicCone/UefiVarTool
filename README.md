<p align="center"><img alt="UEFI Variable Tool (UVT) Logo" src="https://github.com/GeographicCone/UefiVarTool/blob/master/extra/uvt-logo.png?raw=true" width="67%"/></p>

# UEFI Variable Tool (UVT)

**UEFI Variable Tool (UVT)** is a command-line application that runs from the UEFI shell. It can be launched in seconds from any FAT flash drive with no prior machine-specific setup, and lets you view and modify the content of individual UEFI variables at a byte level.

**UVT**'s purpose is to allow changing all the hidden _UEFI (BIOS) Setup_ hardware settings. It is well-suited for situations when custom firmware, such as a modified BIOS that would unhide all the menu forms, cannot be flashed due to restrictive anti-features such as _Boot Guard_ being enabled.

While various utilities have existed for a long time to allow doing just the same, making the functionality as such hardly a novelty, **UVT** aims to make the process as efficient and unencumbered as possible. To that effect:
* It greatly streamlines the command-line **argument syntax**
* It can also work in **scripted mode** to execute an arbitrarily-long series of operations from the **standard input** (_stdin_)
* While in this mode, it allows **defining aliases** to identify data of interest (variable name, offset, and size), and **referencing** these later for read and write operations
* The **output format follows the input**, which means it can be saved to a file and fed back as standard input to restore the **saved settings** later

----

_**UVT** is free software, and full source code is available for anyone to tinker with. It is a quite heavily modified version of [setup_var.efi](https://github.com/datasone/setup_var.efi) by **[@datasone](https://github.com/datasone)**, to whom I originally made some [improvement suggestions](https://github.com/datasone/setup_var.efi/issues/14#issue-1863056272). He [followed up](https://github.com/datasone/setup_var.efi/issues/14#issuecomment-1727910880) on them and even graciously included me as a co-author in his commit, never mind that I had not at that point written a single line of code._

_**@datasone**'s [last-posted version](https://github.com/datasone/setup_var.efi/commit/8c72429113f6fc5e7a4aac63a323d51a2d9f9dd8) seemed at least 90% there but wasn't working for me in a couple of ways. Since he's probably busy with other stuff in his life, I decided it was my turn to contribute something. Hence this fork, which although is completely refactored to the point it might not look very much like the original, would have never been possible without all of **@datasone**'s work, which he generously shared with the world. For that I am eternally grateful to him, and want everyone to know he deserves all the credit as the **original author** of this utility._

## How to Use

The in-application usage information summary is reproduced below for reference:

````
Usage: uvt[.efi] [<Options>] <Op1> [<Op2> [... [<OpN>]]
- or - uvt[.efi] < <InputFile>
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
  uvt -s Lang:0x00 Lang:0x00(4)=0x01020304 Lang:0x00(4)
  Read byte at offset 0, simulate-set the dword (4 bytes), then read again
Example Input File:
  !simulate              # Simulate only, do not perform actual writes
  Language,Lang:0x00(4)  # Define a reference under the alias "Language"
  @Language=0x01020304   # Write to the target referred to by "Language"

<Offset>, <Size> and <Value> can be decimal or hexadecimal: use prefix "0x"
File should be a UTF-16 LE text, UEFI firmware and shell version-dependent
Output saved to a file can be re-used as input again: format is the same
````

### Prerequisites

You need to boot into UEFI shell on the machine where you want to use the utility. This typically involves setting up a FAT flash drive and placing an EFI shell binary under the path `efi/boot/bootx64.efi`. You can then place the **UVT** binary under `efi/tools/uvt.efi` and run it as `uvt` regardless of what the current directory is. More on this in the _Background_ section.

On a broader note, you need to know the variable layout, which is specific to your particular hardware, and possibly even the firmware version. How to obtain this information is also addressed in the _Background_ section.

Separately, it is possible to run **UVT** in an emulator. As this would be mostly of interest to a developer, it is described in the _Building_ section.

### Command Line

There are two ways to use **UVT**. The first one is to run it with command-line arguments.

**UVT** accepts two kinds of command-line arguments: _operations_ and _options_. Arguments are separated with ` ` spaces. There is no limit on the number of arguments.

#### Options

_Options_ start with a `-` (minus) sign and are used to define global-scope settings. Each option has a short and a long form, taking a single `-` and a letter or a double `--` and a keyword respectively. The options are:
* `-f` or `--force` Force-write values where the current values is equal to the new one. The default behavior is to skip such operations, and annotate such entries with an `# Already` comment in the output.
* `-h` or `--help` Shows the usage information. If this option is selected, no other operations will be performed.
* `-r` or `--restart` Reboots the system upon succesful completion. No restart will be performed if any of the operations failed.
* `-s` or `--simulate` If set, no changes will be made to UEFI variables. All the other aspects of the application will still be functioning exactly in the same way. This might be useful for checking what an operation would do, or whether the arguments are syntactically correct. If `-f` or `--force` is specified together with this option, no writing will happen regardless: the simulation takes precedence.

#### Operations

_Operations_ define either reading (querying, or getting) or writing (assigning, or setting) a value. The syntax is:

````
<VarName>[(<VarId>)]:<Offset>[(<Size>)][=<Value>]
````
Where:
* `<VarName>` is the UEFI variable name. It is case-sensitive and mandatory: there is no default.
* `<VarId>` is an optional identifier to distinguish between variables in a situation when two or more share the same name. In the unlikely scenario this happens, the application will automatically list all the variables with the matching name, alongside with their respective identifiers.
* `<Offset>` is the position of data within the variable where the value data starts. Remember the count starts from 0, not 1.
* `<Size>` is the optional size of the variable: it defaults to a single byte, i.e. `(1)`, which can also be specified, although that's unnecessary. The application can write at most 8 bytes (or 64 bits) at a time.
* `<Value>` is the _new_ value to be written at the given offset. The value must fit within the `<Size>` constraint, which is checked. Multi-byte values are little-endian, which means that if you write `0x01` to 4 bytes starting at offset `0x00`, the value of `0x01` will be at the offset of `0x00` and not `0x03`, although if you _read_ these 4 bytes again, the result will also be shown as `0x00000001`. If you are unfamiliar with the concept or do not understand its implications, it's best to write individual bytes, and that's what the vast majority of _UEFI Setup_ settings are anyway. This part, alongside the `=` assignment operator, is optional: if absent, the default action is to query and output the _current_ value.

For example:
* `uvt Lang:0x00` reads the byte value at offset `0x00` in the variable `Lang`
* `uvt -s Lang:0x00(4)=0x01020304` _simulates_ writing a double-word (four-byte) value to an offset starting at `0x00` in the variable `Lang`
* `uvt Lang:0x00(4)` reads again the double word that has just been written with the preceding command

An arbitrary number of command-line operations can be specified. They will be executed in the order entered. An error interrupts the processing of any further operations and arguments, terminating the application.

#### Numerical Values

Any number can be specified as either _decimal_ (base 10) or _hexadecimal_ (base 16). Hexadecimal values should be preceded by `0x` or `0X`, otherwise they will be parsed as decimal. Only digits `0-9` are allowed in decimal values. The additional digits `a-f` and `A-F` in hexadecimal values are case-insensitive.

_Offsets_ and _values_ are output in hexadecimal, while _sizes_ are shown in decimal. When printed, hexadecimal values for _offsets_ will be zero-padded to 2 bytes. _Values_ will be zero-padded to their _size_. The padding does not have to be preserved in input, i.e. you can type `0x1` for a word-sized (two-byte) value, instead of writing `0x0001`.

#### Output

**UVT**'s output follows the same syntax as the input it accepts. This way, nearly everything it spits out can be fed back to it, for example to restore some previously-saved settings.

The application prints out a header as the first thing it does after it launches, which provides some useful information. It might look like that:

````
# UEFI Variable Tool (uvt) Version 0.0.0 @ American Megatrends 5.24 UEFI 2.8
````

This prompt starting with `#` is also a valid comment, which means it will not interfere if you decided to feed the same file back to **UVT**. The three items following the `@` sign are: the firmware _vendor_, _firmware version_ (_major_._minor_) and the _UEFI revision_ (specification version compatibility).

### Input Stream

**UVT**'s other mode of operation is to take an arbitrarily-long list of commands from the standard input (_stdin_). To use the application in this mode, make sure _not_ to provide _any_ command-line arguments, other than the redirection operator, which is however handled by the shell.

#### Redirection

**UVT** does not read or write files directly: it depends on the UEFI shell redirection. This comes with some quirks, which might also be implementation-dependent.

To feed data to the application's _standard input_, use the following shell command:

````shell
uvt < in.txt
````

The file `in.txt` should be properly formatted, as discussed in the next section. You can also save the application's output:

````shell
uvt > out.txt
````

To combine both operations:

````shell
uvt < in.txt > out.txt
````

The quirks mentioned are as follows:
* The ` ` space between the filename and the redirection sign is mandatory: neither `uvt <in.txt` nor `uvt< in.txt` will work.
* ASCII-mode redirects `<a` and `>a` despite being specified in the [UEFI Shell Manual](https://uefi.org/sites/default/files/resources/UEFI_Shell_2_2.pdf) do not seem to work properly in the latest _EFI Shell_ 2.2 (as of August 2023). Do not use them.
* Input redirection via the `|` pipe operator does not seem to work either with built-in commands such as `echo` or `type`. The standard-input stream received by **UVT** is empty.

#### File Format

As the UEFI shell operates internally with UCS-2 encoding, the accepted standard input file format is _Unicode UTF-16 Little-Endian (LE)_: standard _ASCII_ text files will not work. This is a minor inconvenience, although even the _Notepad_ application bundled with Windows can save text in this format, as long as it is specified explicitly.

Any output files produced by a redirection will also be in the same format.

The _Byte Order Mark_ (BOM), mandated by the UTF-16 specification, is optional as far as the application is concerned. In fact, any BOM instances will be filtered out at an early parsing stage.

The input file is split into individual _entries_, which are rows separated by the _Line Feed_ character `LF` or `\n`. Each line can contain at most a single operation and is self-contained, i.e. no operation can span multiple lines. The _Carriage Return_ character `CR` or `\r` may be present and will be discarded if that's the case.

The format to define _operations_ is just the same as for the command-line arguments. _Options_ can also be defined, however with a different syntax (read on). Beyond that, there are also _definitions_, which can be referenced by _operations_, and _comments_.

##### Comments

Comments are marked with the pound sign `#`. Anything to the right of that sign is discarded. Comments do not have to be separate lines, they can appear on the same line as an _operation_, an _option_ or a _definition_:

````
# This is an example comment on a separate line
!simulate # Simulate only, do not write
Lang:0x00 # Retrieve the byte at offset 0 in "Lang"
````

When the input is parsed, after filtering out the comments, an entry is trimmed of any leading and trailing whitespace characters. Entries that end up blank are at this point entirely discarded.

##### Definitions & References

A target for an _operation_, consisting of a _variable name_, _offset_ and, optionally, _size_, can be defined to be referenced elsewhere in the file. The syntax is:

````
<Def>,<VarName>:<Offset>[(<Size>)]
````

Where `<VarName>`, `<Offset>` and `<Size>` have the same interpretation as discussed in the command-line arguments section, and `<Def>` is an identifier that can be reused later to identify the target. For example:

````
Language,Lang:0x00(4)
````

The syntax for _operations_ in the input stream is extended to include the following:

````
@<Def>[=<Value>]
````

The following example illustrates accessing a value by reference for the purposes of reading and writing respectively:

````
@Language
@Language=0x01020304
````

#### Options

Some of the _options_ (excluding usage information) can be defined in the input stream as well but the syntax for that is different. Namely, it's the `!` bang (exclamation mark) followed by the option keyword:

````
!<Option>
````

The available _options_ are `!force`, `!restart` and `!simulate`, and their interpretation is the same as discussed in the command-line arguments section.

## Background

### Setup

To run **UVT** you need to boot into UEFI shell. The most straightforward way of setting it up is to use an empty flash drive you can boot from:

* Start with an empty USB flash drive, formatted to a _FAT16_ or _FAT32_ filesystem
  * It does not actually _have_ to be empty, as long as you ensure any other files don't get in the way of the UEFI boot process
* Download a compiled UEFI shell binary
  * Recent [official releases](https://github.com/tianocore/edk2/releases) do not provide binaries, those in the official repository are hopelessly out of date
  * One option is to use _Arch Linux_'s [edk2-shell](https://archlinux.org/packages/extra/any/edk2-shell/) package, which is regularly updated: you only need the file `usr/share/edk2-shell/x64/Shell_Full.efi` from inside the package archive `edk2-shell-YYYYMMDD-#-any.pkg.tar.zst`
    * The package is compressed with [zstd](https://en.wikipedia.org/wiki/Zstd) and can be decompressed using for example [7-Zip Z-Standard](https://github.com/mcmilk/7-Zip-zstd)
  * Another option is to use a build provided by **[@pbatard](https://github.com/pbatard)** in his [UEFI-Shell](https://github.com/pbatard/UEFI-Shell/releases/) repository, which are updated every half a year: in this case, download the file `UEFI-Shell-2.2-YYH#-RELEASE.iso` which can be opened with any archiving utility, and extract the file `efi/boot/bootx64.efi` from it
* Whichever way you obtained the UEFI shell binary, rename it (if need be) to `bootx64.efi` and place it on the flash drive in the `efi/boot` directory.
* While at it, also create the directory `efi/tools` and put `uvt.efi` downloaded from the [latest release](https://github.com/GeographicCone/UefiVarTool/releases/latest) in it: you're now ready to roll
* Insert the flash drive into a USB port and boot from it, which might involve pressing one of the function keys to override the boot process, as well as possibly disabling _Secure Boot_, if you have it enabled.

#### Automation

You can use a startup script named `startup.nsh` placed in the `efi/boot` directory. An example script would look as follows:
````batch
@echo -off
fs0:
alias -v so "shellopt.efi"
alias -v v "uvt.efi"
so -s -delay 0
v --help
````

You can now refer to `uvt` as `v`, which saves having to type the extra two letters every time. Furthermore, you can place any commands you want to run automatically on startup below in the file.

One remaining annoyance is that the UEFI shell will have you wait five seconds or press a key before it processes the startup script. This behavior can be changed by passing a command-line argument to the shell: however, it's a chicken-and-egg problem since arguments cannot be passed to the shell directly, only by means of an environment variable.

This problem is discussed in detail in **[@fpmurphy](https://github.com/fpmurphy)**'s blog post from a long while ago: [Problems with UEFI Shell Options](https://blog.fpmurphy.com/2012/07/problems-with-uefi-shell-options.html). He also came up with a solution to it, and that is the `shellopt.efi` script that appears in the example `startup.nsh` above. A more recent, updated build has been made available by BIOS developer **ChiChen** in his post: [Passing Parameters to BootX64.efi](https://chichen.tw/post/2020-06-01-pass-parameters-to-bootx64.efi/). If you use this functionality a lot, these five-second delays add up, and you might want to consider this workaround, as cumbersome as it sounds.

### Variable Information

While **UVT** gives you all the means to access and modify the _UEFI Setup_ settings, no matter if they are hidden from the menu, you still have to know what you're looking at, and what can be done with it. This information depends on your specific hardware, and might possibly also change between different firmware (i.e. UEFI BIOS) versions, and has to be figured out separately. Here is a quick summary of the process:

* Obtain the UEFI BIOS image for the device
  * Download it from the manufacturer's website: this is the easiest way, but double-check it's the correct version; the only catch is that updates are often distributed as Windows executable files: you might need a tool such as [InnoExtract](https://github.com/dscharrer/innoextract), [InnoUnp](https://github.com/WhatTheBlock/innounp) or the like, however the mentioned [7-Zip Z-Standard](https://github.com/mcmilk/7-Zip-zstd) might be enough to deal with it
  * Dump it with a software tool, such as Intel's **Flash Programming Tool** (_FPT_ or _FPT64W_), not officially available from _Intel_ but widely redistributed by vendors with a number of UEFI BIOS updates for commercial-grade hardware, and unofficially also obtainable from a number of places, including the [Win-RAID Forums](https://winraid.level1techs.com/t/89959/1): make sure you use the version matching your _Management Engine_ firmware
  * Dump it with a hardware programmer such as the widely-popular **CH341A** armed with a _Pomona 5250_ SOIC-8 clip and a tool like [FlashROM](https://github.com/flashrom/flashrom)
* Either way, once you have obtained the correct BIOS image, open it with [UEFI Tool](https://github.com/LongSoft/UEFITool/releases/) by pressing <kbd>Ctrl</kbd>-<kbd>O</kbd> (command-line _UEFI Extract_ from the same repository can also be used)
* Search for the _SetupUtility_ EFI module by pressing <kbd>Ctrl</kbd>-<kbd>F</kbd>: the best way is to look for it by its GUID: `899407D7-99FE-43D8-9A21-79EC328CAC21` (in rare cases, the data might be stored elsewhere)
* Once located, right-click on the _PE32 Image Section_ under _Setup_ and choose _Extract Body…_ to extract the file `Section_PE32_Image_Setup_Body.efi` (or similarly named)
* Download [IFR Extractor](https://github.com/LongSoft/IFRExtractor-RS/releases/latest) from the same repository, a command-line utility. Run it as follows: `ifrextractor Section_PE32_Image_Setup_Body.efi verbose`
* The resulting _Internal Forms Representation_ dump file `Section_PE32_Image_Setup_Body.efi.0.0.en-US.ifr.txt` already contains all the information you need about all the settings, however it's a bit cryptic. Thus, I recommend running it through the [SlimIFR](https://github.com/GeographicCone/SlimIFR) script by yours truly to streamline the formatting and make it more human-readable:
````shell
node SlimIFR.js Section_PE32_Image_Setup_Body.efi.0.0.en-US.ifr.txt Setup.txt
````
* In the end, you should end up with entries like the one down below, conveniently already exactly in the same format accepted as input by **UVT**:

````
AMITSESetup:0x0040               # Boot: Quiet Boot [0x00 / 0x01]
````

<details><summary>Or if you skipped the last step</summary>

The untransformed end result would look like this:
````
CheckBox Prompt: "Quiet Boot", Help: "Enables or disables Quiet Boot option", QuestionFlags: 0x0, QuestionId: 0x106E, VarStoreId: 0xF013, VarOffset: 0x40, Flags: 0x0, Default: Disabled, MfgDefault: Disabled
  Default DefaultId: 0x0 Value: 1
  Default DefaultId: 0x1 Value: 1
End 
````
</details>

Either way, with all this information at hand, you're now ready to change any hidden settings. Be careful though, changing some of these may brick (or, more likely, _soft_-brick) your hardware. The usual disclaimers apply: if things go south, you're on your own, so make sure to plan for that contingency.

## Building

Once you have the environment set up (if not, read on), building should be fairly straightforward by running the following command in the source directory:

````
CARGO_BUILD_TARGET=x86_64-unknown-uefi cargo build --release
````

Remove the `--release` flag if you want a debug build which is also much faster to produce.

### Environment

**UVT** is written in [Rust](https://www.rust-lang.org/). To build it, you will need `rustc` (compiler), `rustup` (toolchain installer), and `cargo` (package manager and build automation tool). Make sure these are all installed and in the `PATH`. On Windows, you can use [MSys2](https://www.msys2.org/).

You will need to install the appropriate build target first by running: `rustup target add x86_64-unknown-uefi`. And you will also need an Internet connection, since **UVT** has some external dependencies that have to be resolved at build time: the [uefi](https://crates.io/crates/uefi) and [uefi-services](https://crates.io/crates/uefi-services) _crates_, as well as everything they depend on.

### Firmware Emulator

An optional but recommended step is having an emulator set up as well, so that you can immediately run the application as you build it. This is possible with [QEMU](https://www.qemu.org/) and _Open Virtual Machine Firmware_, [OVMF](https://github.com/tianocore/tianocore.github.io/wiki/OVMF), for which the official repository (again) does not offer binary releases: these are helpfully provided by [Gerd Hoffmann](https://www.kraxel.org) and can be downloaded from [his website](https://www.kraxel.org/repos/jenkins/edk2/).

Once you have downloaded _QEMU_ and extracted the _OVMF_ archives, you can run the emulator as follows:

````batch
@echo off
start /min "" "%TOOLS%\MSys64\mingw64\bin\qemu-system-x86_64w.exe" ^
-device isa-debug-exit,iobase=0xf4,iosize=0x04 ^
-device virtio-rng-pci ^
-drive format=raw,file=fat:rw:Filesystem ^
-drive if=pflash,format=raw,readonly=on,file=OVMF_CODE.fd ^
-drive if=pflash,format=raw,file=OVMF_VARS.fd ^
-m 256M ^
-machine q35,accel=kvm:tcg ^
-net none ^
-nodefaults ^
-smp 4 ^
-vga std
````

The above example is a _Windows_ batch file but the _UN*X_ `sh(1)` syntax is similar enough. Adjust the paths as necessary. Before running the emulator, you also need to create the `Filesystem` directory and set it up with the files as described in the _Setup_ section above. Note that the `shellopt` workaround to bypass the five-second wait on boot before executing the `startup.nsh` script does not work in the emulator.

On Windows, you generally want to use the `qemu-system-x86_64w.exe` binary with the trailing `w` which does not keep the console window open as it is running. Alternatively, it is also possible to make _QEMU_ attach itself to a console window it is being executed from with the `-nographic` switch. While running _QEMU_ in this mode, it is good to know that you can press <kbd>Ctrl</kbd>-<kbd>A</kbd>, <kbd>X</kbd> to terminate it at any time.

In the standard, windowed mode it's helpful to be aware that <kbd>Ctrl</kbd>-<kbd>Alt</kbd>-<kbd>F</kbd> toggles full-screen mode, and you can press <kbd>Ctrl</kbd>-<kbd>Alt</kbd>-<kbd>G</kbd> anytime to stop the mouse events from being captured by the client: use this if your cursor has suddenly disappeared.

#### Project Layout

The source files are organized as follows. In the project root directory:

* `Cargo.toml` contains the project metadata and build settings being used by `cargo` and the compiler toolchain
* `Cargo.lock` is an automatically-generated file that stores the information about package dependency versions used by the project; if deleted, it will be regenerated but if the information there changes, the project might no longer build, or it might introduce unpredictable errors in **UVT**'s operations due to changes upstream: consider yourself warned
* `target` is the directory where all the objects and information generated during the build process is stored, alongside the resulting executable in `target/x86_64-uknown-uefi/{debug,release}/uvt.efi`: all of this can be safely deleted at any time
* `src` is where all the source files are located, and the directory is discussed separately below

##### Source Files

The source files (all with the `*.rs` extension) are organized as follows:

* `main.rs` is the main file that provides the entry point and launches all operations

Most of the logic (code) is located in the following three files:

* `firmware.rs` performs UEFI operations such as querying and setting UEFI variables
* `parse.rs` processes command-line and stream (standard) input into data structures
* `string.rs` provides string manipulation routines, including an extension to `CStr16` (UEFI-specific equivalent to `str`)

The following files contain primarily data, with very little code:

* `config.rs` stores configurable parameters together for easy adjustment
* `config/locale_en.rs` stores translateable user interface messages
* `data.rs` defines data types and structures used throughout the application
* `error.rs` allows for error handling in a single centralized manner

The application can easily be translated to other languages by making a copy of `locale_en.rs` as `locale_XX.rs`, where `XX` is a two-letter [ISO 639-1 language code](https://en.wikipedia.org/wiki/List_of_ISO_639-1_codes). The entry for `locale_en` can then be replaced with `locale_XX` in `config.rs`.

## License

* Copyright © 2022 [@datasone](https://github.com/datasone)
* Copyright © 2023 [Piotr Szczepański](https://piotr.szczepanski.name/) ([@GeographicCone](https://github.com/GeographicCone))

**UVT** is _free software_: you can redistribute it and/or modify it under the terms of the [GNU General Public License Version 3](https://www.gnu.org/licenses/gpl-3.0.html#license-text) as published by the [Free Software Foundation](https://www.fsf.org/). The full text of the license is available as `LICENSE.md` in this repository.

**UVT** is a fork of **[setup_var.efi](https://github.com/datasone/setup_var.efi)** based on the [last-posted version](https://github.com/datasone/setup_var.efi/commit/8c72429113f6fc5e7a4aac63a323d51a2d9f9dd8) at the time of the initial release. The original author of this software is **[@datasone](https://github.com/datasone)** who has generously made his work available under the terms of either the _Apache 2.0_ or _MIT)_ license.

I am eternally grateful to **[@datasone](https://github.com/datasone)** for all his work, and implementing the ideas I previously suggested to him. I mostly stepped in to fix the issues that prevented me from using the newest version in his absence. Even if the original did not work for me, once the errors were addressed, I believe it contained about 90% of the current functionality of **UVT** at the time of the initial release. This is why it is extremely important for me to give credit where credit is due. **UVT** as it is would have never been possible without all of **@datasone**'s work, which he generously shared with the world.
For that I am eternally grateful to him, and want everyone to know he deserves all the credit as the **original author** of this utility.

That being said, the source has been completely refactored. As a result, all errors and issues within are mine to own. Please do not bother the original author **@datasone** about any issues you encounter when running **UVT**.

The [original license terms](https://github.com/datasone/setup_var.efi/commit/a535a524a6f5282f84c01e76c13d3cb2f39c14e9) for the portions authored by **@datasone** are reproduced below:

> MIT License
>
> Copyright (c) 2022 datasone
>
> Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
>
> The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
>
> THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

Note that this only applies to the code from [setup_var.efi](https://github.com/datasone/setup_var.efi) in its original repository. **UVT** in its entirety as a fork of it is solely available under the terms of the _GNU General Public License Version 3_, as indicated in the `LICENSE.md` file in this repository.

Please also note that the files in the `extra` directory are not covered by the repository-wide license. In particular, the **UVT** logo is made available under the terms of [CC BY-NC-ND 4.0](http://creativecommons.org/licenses/by-nc-nd/4.0/).

## Version History

### 1.0.0 Initial Public Release

* Refactor and redistribute the code between the source files
* Gather all configurable options in a single source file
* Provide the option for user interface messages to be localized (translated) to different languages
* Fix the issue where the application would try to parse its own name as the first argument and fail unless the `.efi` extension in lower case was explicitly specified as part of the command line
* Fix the issue where the application would never reach the state where it should read and parse the standard input, regardless if standard input was provided
* Fix the issue where no variable identifier or value size could be succesfully entered because the trailing bracket was not stripped from the relevant part of a string, and parsing any of the provided values as a number, whether decimal or hexadecimal, would fail for this reason
* Simplify the input-stream parser by applying prior sanitization at the initial reading
* Change `--write_on_demand` to `--force` (or `-f` in short) and invert the meaning, also change `--reboot` to `--restart`
* Add `--simulate` (or `-s` in short) simulation mode, where no actual write operations are performed
* Change the special characters to use `,` (comma) instead of `:=` (Algol/Pascal-style assignment operator) for _definitions_, `!` instead of `@` for _options_, and `@` instead of `$` for references in _operations_
* Provide a header with application version, as well as firmware vendor, version, and UEFI revision number
* Provide more detailed and meaningful error messages when encountering errors throughout the application, particularly parsing errors
* Dynamically update the application executable image name when displaying usage information
* Add the functionality that references can also be used for read operations, which originally resulted in a (perhaps unintended) parse error
* Update the `uefi` and `uefi_services` external package dependencies to their latest versions
