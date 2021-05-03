//! Rust bindings & interface for the N64 development library "libdragon"
//! by DragonMinded (https://dragonminded.com/n64dev/libdragon/).
//!
//! Module and method documentation taken from the libdragon doxygen
//! documentation at https://dragonminded.com/n64dev/libdragon/doxygen/
#![no_std]

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#![feature(asm)]

mod bindings;

/// Interface to the N64 audio hardware.
///
/// The audio subsystem handles queueing up chunks of audio data for playback
/// using the N64 audio DAC. The audio subsystem handles DMAing chunks of data
/// to the audio DAC as well as audio callbacks when there is room for another
/// chunk to be written. Buffer size is calculated automatically based on the
/// requested audio frequency. The audio subsystem accomplishes this by interfacing
/// with the audio interface (AI) registers.
///
/// Because the audio DAC is timed off of the system clock of the N64, the audio
/// subsystem needs to know what region the N64 is from. This is due to the fact
/// that the system clock is timed differently for PAL, NTSC and MPAL regions.
/// This is handled automatically by the audio subsystem based on settings left
/// by the bootloader.
///
/// Code attempting to output audio on the N64 should initialize the audio subsystem
/// at the desired frequency and with the desired number of buffers using audio_init.
/// More audio buffers allows for smaller chances of audio glitches but means that
/// there will be more latency in sound output. When new data is available to be output,
/// code should check to see if there is room in the output buffers using audio_can_write.
/// Code can probe the current frequency and buffer size using get_frequency() and get_buffer_length()
/// respectively. When there is additional room, code can add new data to the output
/// buffers using write(). Be careful as this is a blocking operation, so if code
/// doesn't check for adequate room first, this function will not return until there
/// is room and the samples have been written. When all audio has been written, code
/// should call close() to shut down the audio subsystem cleanly.
pub mod Audio {
    use cty::*;

    use crate::bindings;

    pub type Frequency = c_int;
    pub type fill_buffer_callback = extern "C" fn(buffer: *mut c_short, numsamples: size_t);

    /// Initialize the audio subsystem.
    ///
    /// This function will set up the AI to play at a given frequency
    /// and allocate a number of back buffers to write data to.
    ///
    /// Note: Before re-initializing the audio subsystem to a new playback frequency, remember to call audio_close.
    pub fn init(frequency: Frequency, numbuffers: i32) {
        unsafe {bindings::audio_init(frequency, numbuffers)}
    }

    /// Set callback function for when the audio buffer is empty and needs more sample data
    pub fn set_buffer_callback(fill_buffer_callback: fill_buffer_callback) {
        unsafe { bindings::audio_set_buffer_callback(fill_buffer_callback) };
    }

    /// Pause or unpause the audio
    pub fn pause(pause: bool) {
        unsafe { bindings::audio_pause(pause) }
    }

    /// Write a chunk of audio data.
    ///
    /// This function takes a chunk of audio data and writes it to an internal
    /// buffer which will be played back by the audio system as soon as room
    /// becomes available in the AI. The buffer should contain stereo interleaved
    /// samples and be exactly audio_get_buffer_length stereo samples long.
    ///
    /// Note: This function will block until there is room to write an audio
    /// sample. If you do not want to block, check to see if there is room
    /// by calling audio_can_write.
    pub fn write_buffer(buffer: &[i16]) {
        unsafe { bindings::audio_write(buffer.as_ptr()) }
    }

    /// Return whether there is an empty buffer to write to.
    ///
    /// This function will check to see if there are any buffers that are not
    /// full to write data to. If all buffers are full, wait until the AI has
    /// played back the next buffer in its queue and try writing again
    pub fn can_write() -> i32 {
        unsafe { return bindings::audio_can_write().extract_inner(); }
    }

    /// Write a chunk of silence.
    ///
    /// This function will write silence to be played back by the audio system.
    /// It writes exactly audio_get_buffer_length stereo samples.
    ///
    /// Note: This function will block until there is room to write an audio sample.
    /// If you do not want to block, check to see if there is room by calling audio_can_write.
    pub fn write_silence() {
        unsafe { bindings::audio_write_silence(); }
    }

    /// Close the audio subsystem.
    ///
    /// This function closes the audio system and cleans up any internal
    /// memory allocated by audio_init.
    pub fn close() {
        unsafe { bindings::audio_close(); }
    }

    /// Get actual frequency of audio playback.
    ///
    /// Returns: Frequency in Hz of the audio playback
    pub fn get_frequency() -> Frequency {
        unsafe { return bindings::audio_get_frequency(); }
    }

    /// Get the number of stereo samples that fit into an allocated buffer.
    ///
    /// Note: To get the number of bytes to allocate, multiply the return by 2 * sizeof( short )
    ///
    /// Returns: The number of stereo samples in an allocated buffer
    pub fn get_buffer_length() -> i32 {
        unsafe { return bindings::audio_get_buffer_length(); }
    }
}

/// Software console emulation for debugging and simple text output.
///
/// Console support is provided as a poor-man's console for simple debugging on the N64.
/// It does not respect common escape sequences and is nonstandard in size. When using the
/// console, code should be careful to make sure that the display system has not been
/// initialized. Similarly, if the display system is needed, code should be sure that the
/// console is not initialized.
///
/// Code wishing to use the console should first initialize the console support in libdragon
/// with init(). Once the console has been initialized, it wil operate in one of two modes.
/// In automatic mode, every write to the console will be immediately displayed on the screen.
/// The console will be scrolled when the buffer fills. In manual mode, the console will only
/// be displayed after calling render(). To set the render mode, use set_render_mode(). To
/// add data to the console, use #printf or #iprintf. To clear the console and reset the scroll,
/// use clear(). Once the console is not needed or when the code wishes to switch to the display
/// subsystem, clear() should be called to cleanly shut down the console support.
pub mod Console {
    use crate::bindings;

    #[repr(i32)]
    pub enum RenderMode {
        RenderManual = 0,
        RenderAutomatic = 1
    }

    pub const CONSOLE_WIDTH: u64 = 64; // Characters per line for console
    pub const CONSOLE_HEIGHT: u64 = 28; // Lines per screen
    pub const TAB_WIDTH: u64 = 4; // Needs to divide evenly into CONSOLE_WIDTH
    pub const HORIZONTAL_PADDING: u64 = 64;
    pub const VERTICAL_PADDING: u64 = 8;

    /// Initialize the console system. This will initialize the
    /// video properly, so a call to the display_init() function is not necessary.
    pub fn init() {
        unsafe { bindings::console_init(); }
    }

    /// Free the console system. This will clean up any dynamic memory that was in use.
    pub fn close() {
        unsafe { bindings::console_close(); }
    }

    /// This sets the render mode of the console. The RenderAutomatic
    /// mode allows console_printf to immediately be placed onto the screen.
    /// This is very similar to a normal console on a unix/windows system.
    /// The RenderManual mode allows console_printf to be buffered, and displayed
    /// at a later date using render(). This is to allow a rendering
    /// interface somewhat analogous to curse's
    pub fn set_render_mode(mode: RenderMode) {
        unsafe { bindings::console_set_render_mode(mode as i32); }
    }

    /// Clear the console and set the virtual cursor back to the top left.
    pub fn clear() {
        unsafe { bindings::console_clear(); }
    }

    /// Render the console to the screen. This should be called when in
    /// manual rendering mode to display the console to the screen.
    /// In automatic mode it is not necessary to call.
    ///
    /// The color that is used to draw the text can be set using set_color().
    pub fn render() {
        unsafe { bindings::console_render(); }
    }
}

/// Controller and accessory interface
pub mod Controller {
    use bitfield::bitfield;
    use cty::*;

    use crate::bindings;

    #[repr(C)]
    pub struct ControllerData {
        pub c: [N64Controller; 4],
        pub gc: [GCController; 4]
    }

    bitfield! {
        #[repr(C)] pub struct N64Controller(u64);
        impl Debug;
        padding_1, _: 63, 48;
        pub err, set_err: 47, 46;
        padding_2, _: 45, 32;
        // Union: 1 {
        pub data, set_data: 31, 0;
        // } 2 {
        pub A, set_A: 31;
        pub B, set_B: 30;
        pub Z, set_Z: 29;
        pub start, set_start: 28;
        pub up, set_up: 27;
        pub down, set_down: 26;
        pub left, set_left: 25;
        pub right, set_right: 24;
        padding_3, _: 23, 22;
        pub L, set_L: 21;
        pub R, set_R: 20;
        pub C_up, set_C_up: 19;
        pub C_down, set_C_down: 18;
        pub C_left, set_C_left: 17;
        pub C_right, set_C_right: 16;
        pub x, set_x: 15, 8;
        pub y, set_y: 7, 0;
        // }
    }

    bitfield! {
        #[repr(C)] pub struct GCController(u64);
        impl Debug;
        // Union: 1 {
        pub data, set_data: 63, 0;
        // } 2 {
        pub err, set_err: 63, 62;
        pub origin_unchecked, set_origin_unchecked: 61;
        pub start, set_start: 60;
        pub y, set_y: 59;
        pub x, set_x: 58;
        pub b, set_b: 57;
        pub a, set_a: 56;
        unused2, _: 55;
        pub l, set_l: 54;
        pub r, set_r: 53;
        pub z, set_z: 52;
        pub up, set_up: 51;
        pub down, set_down: 50;
        pub right, set_right: 49;
        pub left, set_left: 48;
        pub stick_x, set_stick_x: 47, 40;
        pub stick_y, set_stick_y: 39, 32;
        pub cstick_x, set_cstick_x: 31, 24;
        pub cstick_y, set_cstick_y: 23, 16;
        pub analog_l, set_analog_l: 15, 8;
        pub analog_r, set_analog_r: 7, 0;
        // }
    }

    #[repr(C)]
    pub struct ControllerOriginData {
        pub gc: [GCControllerOrigin; 4]
    }

    #[repr(C)]
    pub struct GCControllerOrigin {
        pub data: GCController,
        pub deadzone0: uint8_t,
        pub deadzone1: uint8_t
    }

    #[repr(i32)]
    pub enum AccessoryType {
        None = 0,      // No accessory present
        MemPak = 1,    // Controller pack present
        RumblePak = 2, // Rumble Pak present
        VRU = 3,       // VRU present
    }

    #[repr(i32)]
    pub enum EEPROMType {
        None = 0,
        _4K = 1,
        _16K = 2
    }

    #[repr(i32)]
    pub enum SIError {
        None = 0x0,       // No error occured
        BadCommand = 0x1, // Command not recognized or malformed
        NotPresent = 0x2  // Controller not present
    }

    #[repr(i32)]
    pub enum DPadDirection {
        R = 0,  // Right
        UR = 1, // Up + right
        U = 2,  // Up
        UL = 3, // Up + left
        L = 4,  // Left
        DL = 5, // Down + left
        D = 6,  // Down
        DR = 7  // Down + right
    }

    #[repr(i32)]
    pub enum MemPakResult {
        Success = 0,
        OutOfRange = -1,
        NoMemPak = -2,
        InvalidMemPakData = -3
    }

    #[repr(i32)]
    pub enum ControllerNum {
        Controller1 = 0,
        Controller2 = 1,
        Controller3 = 2,
        Controller4 = 3
    }

    pub const CONTROLLER_1_INSERTED: u64 = 0xF000;
    pub const CONTROLLER_2_INSERTED: u64 = 0x0F00;
    pub const CONTROLLER_3_INSERTED: u64 = 0x00F0;
    pub const CONTROLLER_4_INSERTED: u64 = 0x000F;

    /// Initialize the controller subsystem.
    pub fn init() {
        unsafe { bindings::controller_init(); }
    }

    /// Read the controller button status immediately and return results
    /// to data. If calling this function, one should not also call
    /// scan_controllers() as this does not update the
    /// internal state of controllers.
    pub fn read_controller_data(data_out: &mut ControllerData) {
        unsafe { bindings::controller_read(data_out); }
    }

    /// Read the controller button status immediately and return results to data.
    pub fn read_gc_controller_data(data_out: &mut ControllerData, rumble: [u8; 4]) {
        unsafe { bindings::controller_read_gc(data_out, &rumble); }
    }

    /// This returns the values set on power up, or the values the user requested
    /// by reseting the controller by holding X-Y-start. Apps should use these as
    /// the center stick values. The meaning of the two deadzone values is unknown.
    pub fn read_gc_origin_controller_data(data_out: &mut ControllerOriginData) {
        unsafe { bindings::controller_read_gc_origin(data_out); }
    }

    /// Queries the controller interface and returns a bitmask specifying
    /// which controllers are present. See CONTROLLER_1_INSERTED,
    /// CONTROLLER_2_INSERTED, CONTROLLER_3_INSERTED and CONTROLLER_4_INSERTED.
    pub fn get_controllers_present() -> i32 {
        unsafe { return bindings::get_controllers_present(); }
    }

    /// Queries the controller interface and returns a bitmask specifying
    /// which controllers have recognized accessories present.
    /// See CONTROLLER_1_INSERTED, CONTROLLER_2_INSERTED, CONTROLLER_3_INSERTED
    /// and CONTROLLER_4_INSERTED.
    pub fn get_accessories_present(data: &mut ControllerData) -> i32 {
        unsafe { return bindings::get_accessories_present(data); }
    }

    /// Scan the four controller ports and calculate the buttons state.
    /// This must be called before calling get_keys_down(), get_keys_up(),
    /// get_keys_held(), get_keys_pressed() or get_dpad_direction().
    pub fn scan_controllers() {
        unsafe { bindings::controller_scan(); }
    }

    /// Return keys pressed since last detection. This returns a standard
    /// ControllerData struct identical to read_controller_data(). However,
    /// buttons are only set if they were pressed down since the last controller_scan.
    pub fn get_keys_down() -> ControllerData {
        unsafe { return bindings::get_keys_down(); }
    }

    /// Return keys released since last detection. This returns a standard
    /// ControllerData struct identical to read_controller_data(). However,
    /// buttons are only set if they were released since the last controller_scan.
    pub fn get_keys_up() -> ControllerData {
        unsafe { return bindings::get_keys_up(); }
    }

    /// Return keys held since last detection. This returns a standard
    /// ControllerData struct identical to read_controller_data(). However,
    /// buttons are only set if they were held since the last controller_scan.
    pub fn get_keys_held() -> ControllerData {
        unsafe { return bindings::get_keys_held(); }
    }

    /// This function works identically to read_controller_data() except for
    /// it is safe to call when using scan_controllers().
    pub fn get_keys_pressed() -> ControllerData {
        unsafe { return bindings::get_keys_pressed(); }
    }

    /// Return the direction of the DPAD specified in controller.
    /// Follows standard polar coordinates, where 0 = 0, pi/4 = 1,
    /// pi/2 = 2, etc... Returns -1 when not pressed. Must be used in
    /// conjunction with Controller::scan_controllers()
    pub fn get_dpad_direction(controller: ControllerNum) -> DPadDirection {
        unsafe {
            return match bindings::get_dpad_direction(controller as i32) {
                0 => DPadDirection::R,
                1 => DPadDirection::UR,
                2 => DPadDirection::U,
                3 => DPadDirection::UL,
                4 => DPadDirection::L,
                5 => DPadDirection::DL,
                6 => DPadDirection::D,
                7 => DPadDirection::DR,
                bad => panic!("Invalid result from Controller::get_dpad_direction(): {}", bad)
            };
        }
    }

    /// Given a controller and an address, read 32 bytes from a mempak and return them in data.
    pub fn read_mempak_address(controller: ControllerNum, address: u16, data_out: &mut [u8]) -> MemPakResult {
        unsafe {
            return match bindings::read_mempak_address(controller as i32, address, data_out.as_mut_ptr()) {
                0 => MemPakResult::Success,
                -1 => MemPakResult::OutOfRange,
                -2 => MemPakResult::NoMemPak,
                -3 => MemPakResult::InvalidMemPakData,
                bad => panic!("Invalid result from Controller::read_mempak_address(): {}", bad)
            };
        }
    }

    /// Given a controller and an address, write 32 bytes to a mempak from data.
    pub fn write_mempak_address(controller: ControllerNum, address: u16, data_in: &mut [u8]) -> MemPakResult {
        unsafe {
            return match bindings::write_mempak_address(controller as i32, address, data_in.as_mut_ptr()) {
                0 => MemPakResult::Success,
                -1 => MemPakResult::OutOfRange,
                -2 => MemPakResult::NoMemPak,
                -3 => MemPakResult::InvalidMemPakData,
                bad => panic!("Invalid result from Controller::write_mempak_address(): {}", bad)
            };
        }
    }

    /// Given a controller, identify the particular accessory type inserted.
    pub fn identify_accessory(controller: ControllerNum) -> AccessoryType {
        unsafe {
            return match bindings::identify_accessory(controller as i32) {
                0 => AccessoryType::None,
                1 => AccessoryType::MemPak,
                2 => AccessoryType::RumblePak,
                3 => AccessoryType::VRU,
                bad => panic!("Invalid result from Controller::identify_accessory(): {}", bad)
            };
        }
    }

    /// Turn rumble on for a particular controller.
    pub fn rumble_start(controller: ControllerNum) {
        unsafe { bindings::rumble_start(controller as i32); }
    }

    /// Turn rumble off for a particular controller.
    pub fn rumble_stop(controller: ControllerNum) {
        unsafe { bindings::rumble_stop(controller as i32); }
    }

    /// Send an arbitrary command to a controller and receive arbitrary data back <br>
    /// Note: In the original library, the bytesout and bytesin are swapped for some reason.
    ///
    /// controller - The controller to send the command to <br>
    /// command - The command byte to send <br>
    /// bytes_in - The number of parameter bytes the command requires <br>
    /// bytes_out - The number of result bytes expected <br>
    /// input - The parameter bytes to send with the command <br>
    /// output - The result bytes returned by the operation
    pub fn execute_raw_command(controller: ControllerNum, command: i32, bytes_in: i32, bytes_out: i32, input: &mut [u8], output: &mut [u8]) {
        unsafe {
            bindings::execute_raw_command(
                controller as i32,
                command,
                bytes_in,
                bytes_out,
                input.as_mut_ptr(),
                output.as_mut_ptr()
            );
        }
    }

    /// Probe the EEPROM to see if it exists on this cartridge.
    pub fn eeprom_present() -> EEPROMType {
        unsafe {
            return match bindings::eeprom_present() {
                0 => EEPROMType::None,
                1 => EEPROMType::_4K,
                2 => EEPROMType::_16K,
                bad => panic!("Invalid result from Controller::eeprom_present(): {}", bad)
            };
        }
    }

    /// Read a block from EEPROM.
    ///
    /// Parameters:
    ///
    /// block - Block to read data from. The N64 accesses eeprom in 8 byte blocks.
    ///
    /// buffer - Buffer to place the eight bytes read from EEPROM.
    pub fn eeprom_read(block: i32) -> [u8; 8] {
        let buffer: &mut [u8; 8] = &mut [0; 8];

        unsafe {
            bindings::eeprom_read(block, buffer.as_mut_ptr());
        }

        return *buffer;
    }

    /// Write a block to EEPROM.
    ///
    /// Parameters:
    ///
    /// block - Block to write data to. The N64 accesses eeprom in 8 byte blocks.
    ///
    /// data - Eight bytes of data to write to block specified
    pub fn eeprom_write(block: i32, data: &[u8; 8]) {
        unsafe { bindings::eeprom_write(block, data.as_ptr()); }
    }
}

/// Managed mempak interface
pub mod MemoryPak {
    use cty::*;

    use crate::{Controller::ControllerNum, bindings};

    #[repr(C)]
    pub struct EntryStructure {
        pub vendor: uint32_t,
        pub game_id: uint16_t,
        pub inode: uint16_t,
        pub region: uint8_t,
        pub blocks: uint8_t,
        pub valid: uint8_t,
        pub entry_id: uint8_t,
        pub name: [c_char; 19]
    }

    #[repr(i32)]
    pub enum DeleteEntryResult {
        DeletedSuccessfully = 0,
        InvalidEntry = -1,
        BadMemPak = -2 // Or mempak isn't present
    }

    #[repr(i32)]
    pub enum FormatResult {
        FormattedSuccessfully = 0,
        BadMemPak = -2 // Or not present
    }

    #[repr(i32)]
    pub enum GetEntryResult {
        ReadSuccessfully = 0,
        BadEntry = -1, // Out of bounds or entry_data is null
        BadMemPak = -2, // Or isn't present
    }

    #[repr(i32)]
    pub enum ReadEntryDataResult {
        ReadSuccessfully = 0,
        BadEntry = -1, // Out of bounds or corrupted
        BadMemPak = -2, // Or not present
        DataUnreadable = -3
    }

    #[repr(i32)]
    pub enum ReadSectorResult {
        ReadSuccessfully = 0,
        BadSector = -1, // Sector out of bounds or sector_data is null
        ErrorRead = -2 // Error reading part of a sector
    }

    #[repr(i32)]
    pub enum ValidateResult {
        Valid = 0,
        NotPresent = -2, // Or couldn't be read
        BadMemPak = -3 // Or unformatted
    }

    #[repr(i32)]
    pub enum WriteEntryDataResult {
        WrittenSuccessfully = 0,
        InvalidParameter = -1, // Or note has no length
        BadMemPak = -2, // Or isn't present
        WriteError = -3,
        NotEnoughSpace = -4,
        TOCFull = -5, // Not enough room in the TOC to add a new entry
    }

    #[repr(i32)]
    pub enum WriteSectorResult {
        WrittenSuccessfully = 0,
        BadSector = -1, // Out of bounds or sector_data is null
        WriteError = -2
    }

    pub const MEMPAK_BLOCK_SIZE: u64 = 256;  // Size in bytes of a mempak block
    pub const BLOCK_EMPTY: u64 = 0x03;       // Block is empty
    pub const BLOCK_LAST: u64 = 0x01;        // Last block in the note
    pub const BLOCK_VALID_FIRST: u64 = 0x05; // First valid block that can contain user data
    pub const BLOCK_VALID_LAST: u64 = 0x7F;  // Last valid block that can contain user data

    /// This will read a sector from a mempak. Sectors on mempaks are always 256 bytes in size.
    pub fn read_sector(controller: ControllerNum, sector: i32, sector_data_out: &mut [u8]) -> ReadSectorResult {
        unsafe {
            return match bindings::read_mempak_sector(controller as i32, sector, sector_data_out.as_mut_ptr()) {
                0 => ReadSectorResult::ReadSuccessfully,
                -1 => ReadSectorResult::BadSector,
                -2 => ReadSectorResult::ErrorRead,
                bad => panic!("Invalid result from MemPak::read_sector(): {}", bad)
            };
        }
    }

    /// This will write a sector to a mempak. Sectors on mempaks are always 256 bytes in size.
    pub fn write_sector(controller: ControllerNum, sector: i32, sector_data_in: &mut [u8]) -> WriteSectorResult {
        unsafe {
            return match bindings::write_mempak_sector(controller as i32, sector, sector_data_in.as_mut_ptr()) {
                0 => WriteSectorResult::WrittenSuccessfully,
                -1 => WriteSectorResult::BadSector,
                -2 => WriteSectorResult::WriteError,
                bad => panic!("Invalid result from MemPak::write_sector(): {}", bad)
            };
        }
    }

    /// This function will return whether the mempak in a particular controller is formatted and valid.
    pub fn validate_mempak(controller: ControllerNum) -> ValidateResult {
        unsafe {
            return match bindings::validate_mempak(controller as i32) {
                0 => ValidateResult::Valid,
                -2 => ValidateResult::NotPresent,
                -3 => ValidateResult::BadMemPak,
                bad => panic!("Invalid result from MemPak::validate_mempak(): {}", bad)
            };
        }
    }

    /// Note that a block is identical in size to a sector. To calculate the
    /// number of bytes free, multiply the return of this function by MEMPAK_BLOCK_SIZE.
    ///
    /// Returns the number of blocks free or a negative number on failure.
    pub fn get_free_space(controller: ControllerNum) -> i32 {
        unsafe { return bindings::get_mempak_free_space(controller as i32); }
    }

    /// Given an entry index (0-15), return the entry as found on the mempak.<br>
    /// If the entry is blank or invalid, the valid flag is cleared.
    pub fn get_entry(controller: ControllerNum, entry: i32, entry_data: &mut EntryStructure) -> GetEntryResult {
        unsafe {
            return match bindings::get_mempak_entry(controller as i32, entry, entry_data) {
                0 => GetEntryResult::ReadSuccessfully,
                -1 => GetEntryResult::BadEntry,
                -2 => GetEntryResult::BadMemPak,
                bad => panic!("Invalid result from MemPak::get_mempak_entry(): {}", bad)
            };
        }
    }

    /// Formats a mempak. Should only be done to wipe a mempak or to initialize
    /// the filesystem in case of a blank or corrupt mempak.
    pub fn format_mempak(controller: ControllerNum) -> FormatResult {
        unsafe {
            return match bindings::format_mempak(controller as i32) {
                0 => FormatResult::FormattedSuccessfully,
                -2 => FormatResult::BadMemPak,
                bad => panic!("Invalid result from MemPak::format_mempak(): {}", bad)
            };
        }
    }

    /// Given a valid mempak entry fetched by get_mempak_entry, retrieves the contents of the entry.
    /// The calling function must ensure that enough room is available in the passed in buffer for
    /// the entire entry. The entry structure itself contains the number of blocks used to store the
    /// data which can be multiplied by MEMPAK_BLOCK_SIZE to calculate the size of the buffer needed.
    pub fn read_entry_data(controller: ControllerNum, entry: &mut EntryStructure, data_out: &mut [u8]) -> ReadEntryDataResult {
        unsafe {
            return match bindings::read_mempak_entry_data(controller as i32, entry, data_out.as_mut_ptr()) {
                0 => ReadEntryDataResult::ReadSuccessfully,
                -1 => ReadEntryDataResult::BadEntry,
                -2 => ReadEntryDataResult::BadMemPak,
                -3 => ReadEntryDataResult::DataUnreadable,
                bad => panic!("Invalid result from MemPak::read_entry_data(): {}", bad)
            };
        }
    }

    /// Given a mempak entry structure with a valid region, name and block count,
    /// writes the entry and associated data to the mempak. This function will not
    /// overwrite any existing user data. To update an existing entry, use delete_entry()
    /// followed by write_entry_data() with the same entry structure.
    pub fn write_entry_data(controller: ControllerNum, entry: &mut EntryStructure, data_in: &mut [u8]) -> WriteEntryDataResult {
        unsafe {
            return match bindings::write_mempak_entry_data(controller as i32, entry, data_in.as_mut_ptr()) {
                0 => WriteEntryDataResult::WrittenSuccessfully,
                -1 => WriteEntryDataResult::InvalidParameter,
                -2 => WriteEntryDataResult::BadMemPak,
                -3 => WriteEntryDataResult::WriteError,
                -4 => WriteEntryDataResult::NotEnoughSpace,
                -5 => WriteEntryDataResult::TOCFull,
                bad => panic!("Invalid result from MemPak::write_entry_data(): {}", bad)
            };
        }
    }

    /// Given a valid mempak entry fetched by get_entry() -- removes the entry and frees all associated blocks.
    pub fn delete_entry(controller: ControllerNum, entry: &mut EntryStructure) -> DeleteEntryResult {
        unsafe {
            return match bindings::delete_mempak_entry(controller as i32, entry) {
                0 => DeleteEntryResult::DeletedSuccessfully,
                -1 => DeleteEntryResult::InvalidEntry,
                -2 => DeleteEntryResult::BadMemPak,
                bad => panic!("Invalid result from MemPak::delete_entry(): {}", bad)
            };
        }
    }
}

/// The Transfer Pak interface allows the ROM and Save files of gameboy and
/// gameboy color cartridges connected to the system to be accessed. Each
/// time you want to access a transfer pak, first call init() to boot up
/// the accessory and ensure that it is in working order. set_power() and
/// set_access() can also be called directly if you need to put the pak in a
/// certain mode and you can verify for youself that the pak is ready for IO
/// by calling get_status() and inspecting the bits.
///
/// To read in the connected gameboy cartridge's header, call get_cartridge_header()
/// which provides you with a struct defining each of the fields. Pass this
/// through to check_header() to verify that the header checksum adds up and
/// the data have been read correctly.
///
/// read() and write() do what you expect, switching banks as needed.
///
/// Whenever not using the transfer pak, it's recommended to power it off by
/// calling set_power(false).
pub mod TransferPak {
    use cty::*;

    use crate::{Controller::ControllerNum, bindings};

    #[repr(C)]
    pub struct GameboyCartridgeHeader {
        pub entry_point: [uint8_t; 4],
        pub logo: [uint8_t; 48],
        pub unnamed_1: GBCTitle,
        pub new_licensee_code: uint16_t,
        pub is_sgb_supported: bool,
        pub cartridge_type: uint8_t,
        pub rom_size_code: uint8_t,
        pub ram_size_code: uint8_t,
        pub destination_code: uint8_t,
        pub old_licensee_code: uint8_t,
        pub version_number: uint8_t,
        pub header_checksum: uint8_t,
        pub global_checksum: uint16_t,
        pub overflow: [uint8_t; 16]
    }

    #[repr(C)]
    pub enum GBCTitle {
        title([uint8_t; 16]),
        old_title(OldTitle),
        new_title(NewTitle)
    }

    #[repr(C)]
    pub struct OldTitle {
        pub title: [uint8_t; 15],
        pub gbc_support: GBCSupportType
    }

    #[repr(C)]
    pub struct NewTitle {
        pub title: [uint8_t; 11],
        pub manufacturer_code: [uint8_t; 4],
        pub gbc_support: GBCSupportType
    }

    #[repr(u8)]
    pub enum GBCSupportType {
        GBC_NOT_SUPPORTED = 0x00,
        GBC_DMG_SUPPORTED = 0x80,
        GBC_ONLY_SUPPORTED = 0xC0
    }

    #[repr(i32)]
    pub enum TPakError {
        Success = 0,
        InvalidArgument = -1,
        NoTPak = -2,
        NoController = -3,
        UnknownBehaviour = -4,
        NoCartridge = -5,
        AddressOverflow = -6
    }

    #[repr(u8)]
    pub enum TPakStatus {
        Ready = 0x01,
        WasReset = 0x04,
        IsResetting = 0x08,
        Removed = 0x40,
        Powered = 0x80
    }

    /// Prepare transfer pak for I/O.
    ///
    /// Powers on the transfer pak and sets access mode to allow I/O to gameboy cartridge.
    /// Will also perform a series of checks to confirm transfer pak can be accessed reliably.
    pub fn init(controller: ControllerNum) -> TPakError {
        unsafe {
            return match bindings::tpak_init(controller as i32) {
                0 => TPakError::Success,
                -1 => TPakError::InvalidArgument,
                -2 => TPakError::NoTPak,
                -3 => TPakError::NoController,
                -4 => TPakError::UnknownBehaviour,
                -5 => TPakError::NoCartridge,
                -6 => TPakError::AddressOverflow,
                bad => panic!("Invalid result from TransferPak::init(): {}", bad)
            };
        }
    }

    /// Set transfer pak or gb cartridge controls/flags.
    ///
    /// Helper to set a transfer pak status or control setting. Be aware that for simplicity's sake,
    /// this writes the same value 32 times, and should therefore not be used for updating individual
    /// bytes in Save RAM.
    pub fn set_value(controller: ControllerNum, address: u16, value: u8) -> TPakError {
        unsafe {
            return match bindings::tpak_set_value(controller as i32, address, value) {
                0 => TPakError::Success,
                -1 => TPakError::InvalidArgument,
                -2 => TPakError::NoTPak,
                -3 => TPakError::NoController,
                -4 => TPakError::UnknownBehaviour,
                -5 => TPakError::NoCartridge,
                -6 => TPakError::AddressOverflow,
                bad => panic!("Invalid result from TransferPak::set_value(): {}", bad)
            };
        }
    }

    /// Change transfer pak banked memory.
    ///
    /// Change the bank of address space that is available between transfer pak addresses 0xC000 and 0xFFFF
    pub fn set_bank(controller: ControllerNum, bank: i32) -> TPakError {
        unsafe {
            return match bindings::tpak_set_bank(controller as i32, bank) {
                0 => TPakError::Success,
                -1 => TPakError::InvalidArgument,
                -2 => TPakError::NoTPak,
                -3 => TPakError::NoController,
                -4 => TPakError::UnknownBehaviour,
                -5 => TPakError::NoCartridge,
                -6 => TPakError::AddressOverflow,
                bad => panic!("Invalid result from TransferPak::set_bank(): {}", bad)
            };
        }
    }

    /// Toggle transfer pak power state.
    pub fn set_power(controller: ControllerNum, power_state: bool) -> TPakError {
        unsafe {
            return match bindings::tpak_set_power(controller as i32, power_state) {
                0 => TPakError::Success,
                -1 => TPakError::InvalidArgument,
                -2 => TPakError::NoTPak,
                -3 => TPakError::NoController,
                -4 => TPakError::UnknownBehaviour,
                -5 => TPakError::NoCartridge,
                -6 => TPakError::AddressOverflow,
                bad => panic!("Invalid result from TransferPak::set_power(): {}", bad)
            };
        }
    }

    /// Set transfer pak access mode.
    pub fn set_access(controller: ControllerNum, access_state: bool) -> TPakError {
        unsafe {
            return match bindings::tpak_set_access(controller as i32, access_state) {
                0 => TPakError::Success,
                -1 => TPakError::InvalidArgument,
                -2 => TPakError::NoTPak,
                -3 => TPakError::NoController,
                -4 => TPakError::UnknownBehaviour,
                -5 => TPakError::NoCartridge,
                -6 => TPakError::AddressOverflow,
                bad => panic!("Invalid result from TransferPak::set_access(): {}", bad)
            };
        }
    }

    /// Gets transfer pak status flags.
    ///
    /// Return values: <br>
    /// Bit 0: Access mode - must be 1 to communicate with a cartridge <br>
    /// Bit 2: Reset status - if set, indicates that the cartridge is in the process of booting up/resetting <br>
    /// Bit 3: Reset detected - Indicates that the cartridge has been reset since the last IO <br>
    /// Bit 6: Cartridge presence - if not set, there is no cartridge in the transfer pak. <br>
    /// Bit 7: Power mode - a 1 indicates there is power to the transfer pak.
    pub fn get_status(controller: ControllerNum) -> u8 {
        unsafe {
            return bindings::tpak_get_status(controller as i32);
        }
    }

    /// Reads a gameboy cartridge header in to memory.
    pub fn get_cartridge_header(controller: ControllerNum, header_out: &mut GameboyCartridgeHeader) -> TPakError {
        unsafe {
            return match bindings::tpak_get_cartridge_header(controller as i32, header_out) {
                0 => TPakError::Success,
                -1 => TPakError::InvalidArgument,
                -2 => TPakError::NoTPak,
                -3 => TPakError::NoController,
                -4 => TPakError::UnknownBehaviour,
                -5 => TPakError::NoCartridge,
                -6 => TPakError::AddressOverflow,
                bad => panic!("Invalid result from TransferPak::get_cartridge_header(): {}", bad)
            };
        }
    }

    /// Verify gb cartridge header.
    ///
    /// This will help you verify that the tpak is connected and working properly.
    pub fn check_header(header: &mut GameboyCartridgeHeader) -> bool {
        unsafe { return bindings::tpak_check_header(header); }
    }

    /// Write data from a buffer to a gameboy cartridge.
    ///
    /// Save RAM is located between gameboy addresses 0xA000 and 0xBFFF, which is in the transfer pak's
    /// bank 2. This function does not account for cartridge bank switching, so to switch between MBC1
    /// RAM banks, for example, you'll need to switch to Tpak bank 1, and write to address 0xE000, which
    /// translates to address 0x6000 on the gameboy.
    pub fn write(controller: ControllerNum, address: u16, data_in: &mut [u8], size: u16) -> TPakError {
        unsafe {
            return match bindings::tpak_write(controller as i32, address, data_in.as_mut_ptr(), size) {
                0 => TPakError::Success,
                -1 => TPakError::InvalidArgument,
                -2 => TPakError::NoTPak,
                -3 => TPakError::NoController,
                -4 => TPakError::UnknownBehaviour,
                -5 => TPakError::NoCartridge,
                -6 => TPakError::AddressOverflow,
                bad => panic!("Invalid result from TransferPak::write(): {}", bad)
            };
        }
    }

    /// Read data from gameboy cartridge to a buffer.
    pub fn read(controller: ControllerNum, address: u16, buffer_out: &mut [u8], size: u16) -> TPakError {
        unsafe {
            return match bindings::tpak_read(controller as i32, address, buffer_out.as_mut_ptr(), size) {
                0 => TPakError::Success,
                -1 => TPakError::InvalidArgument,
                -2 => TPakError::NoTPak,
                -3 => TPakError::NoController,
                -4 => TPakError::UnknownBehaviour,
                -5 => TPakError::NoCartridge,
                -6 => TPakError::AddressOverflow,
                bad => panic!("Invalid result from TransferPak::read(): {}", bad)
            };
        }
    }
}

pub mod Display {
    use cty::*;

    use crate::bindings;

    #[repr(C)]
    pub enum Resolution {
        RESOLUTION_320x240,
        RESOLUTION_640x480,
        RESOLUTION_256x240,
        RESOLUTION_512x480,
        RESOLUTION_512x240,
        RESOLUTION_640x240
    }

    #[repr(C)]
    pub enum BitDepth {
        DEPTH_16_BPP,
        DEPTH_32_BPP
    }

    #[repr(C)]
    pub enum Gamma {
        GAMMA_NONE,
        GAMMA_CORRECT,
        GAMMA_CORRECT_DITHER
    }

    #[repr(C)]
    pub enum AntiAlias {
        ANTIALIAS_OFF,
        ANTIALIAS_RESAMPLE,
        ANTIALIAS_RESAMPLE_FETCH_NEEDED,
        ANTIALIAS_RESAMPLE_FETCH_ALWAYS
    }

    pub type DisplayContext = c_int;

    /// Initialize video system. This sets up a double or triple buffered drawing surface
    /// which can be blitted or rendered to using software or hardware.
    pub fn init(res: Resolution, bitdepth: BitDepth, no_buffers: u32, gamma: Gamma, aa: AntiAlias) {
        unsafe { bindings::display_init(res, bitdepth, no_buffers, gamma, aa); }
    }

    /// Grab a display context that is safe for drawing. If none is available then this
    /// will return 0. Do not check out more than one display context at a time.
    pub fn lock() -> DisplayContext {
        unsafe { return bindings::display_lock(); }
    }

    /// Display a valid DisplayContext to the screen on the next vblank.
    /// Display contexts should be locked via lock().
    pub fn show(disp: DisplayContext) {
        unsafe { bindings::display_show(disp); }
    }

    /// Close a display and free buffer memory associated with it.
    pub fn close() {
        unsafe { bindings::display_close(); }
    }
}

/// DMA functionality for transfers between cartridge space and RDRAM.
///
/// The DMA controller is responsible for handling block and word accesses from
/// the cartridge domain. Because of the nature of the catridge interface, code
/// cannot use memcpy or standard pointer accesses on memory mapped to the catridge.
/// Consequently, the peripheral interface (PI) provides a DMA controller for accessing data.
///
/// The DMA controller requires no initialization. Using dma_read() and dma_write() will
/// allow reading from the cartridge and writing to the cartridge respectively in block
/// mode. io_read() and io_write() will allow a single 32-bit integer to be read from
/// or written to the cartridge. These are especially useful for manipulating registers
/// on a cartridge such as a gameshark. Code should never make raw 32-bit reads or writes
/// in the cartridge domain as it could collide with an in-progress DMA transfer or run
/// into caching issues.
pub mod DMA {
    use volatile::Volatile;

    use crate::bindings;

    /// Write to a peripheral.
    ///
    /// This function should be used when writing to the cartridge.
    pub fn write(ram_address_in: *mut (), pi_address: u32, length: u32) {
        unsafe { bindings::dma_write(ram_address_in.cast(), pi_address, length); }
    }

    /// Read from a peripheral.
    ///
    /// This function should be used when reading from the cartridge.
    pub fn read(ram_address_out: *mut (), pi_address: u32, length: u32) {
        unsafe { bindings::dma_read(ram_address_out.cast(), pi_address, length); }
    }

    /// Return whether the DMA controller is currently busy.
    ///
    /// Returns: nonzero if the DMA controller is busy or 0 otherwise
    pub fn get_busy() -> Volatile<i32> {
        unsafe { return bindings::dma_busy(); }
    }

    /// Read a 32 bit integer from a peripheral.
    pub fn io_read(pi_address: u32) -> u32 {
        unsafe { return bindings::io_read(pi_address); }
    }

    /// Write a 32 bit integer to a peripheral.
    pub fn io_write(pi_address: u32, data: u32) {
        unsafe { return bindings::io_write(pi_address, data); }
    }
}

/// DragonFS filesystem implementation and newlib hooks.
///
/// DragonFS is a read only ROM filesystem for the N64. It provides an
/// interface that homebrew developers can use to load resources from
/// cartridge space that were not available at compile time. This can mean
/// sprites or other game assets, or the filesystem can be appended at a
/// later time if the homebrew developer wishes end users to be able to
/// insert custom levels, music or other assets. It is loosely based off
/// of FAT with consideration into application and limitations of the N64.
///
/// The filesystem can be generated using 'mkdfs' which is included in the
/// 'tools' directory of libdragon. Due to the read-only nature, DFS does
/// not support empty files or empty directories. Attempting to create a
/// filesystem with either of these using 'mkdfs' will result in an error.
/// If a filesystem contains either empty files or empty directories, the
/// result of manipulating the filesystem is undefined.
///
/// DragonFS does not support writing, renaming or symlinking of files.
/// It supports only file and directory types.
///
/// DFS files have a maximum size of 16,777,216 bytes. Directories can have
/// an unlimited number of files in them. Each token (separated by a / in
/// the path) can be 243 characters maximum. Directories can be 100 levels
/// deep at maximum. There can be 4 files open simultaneously.
///
/// When DFS is initialized, it will register itself with newlib using
/// 'rom:/' as a prefix. Files can be accessed either with standard POSIX
/// functions and the 'rom:/' prefix or with DFS API calls and no prefix.
/// Files can be opened using both sets of API calls simultaneously as long
/// as no more than four files are open at any one time.
pub mod DragonFS {
    use cstr_core::CString;
    use cty::*;

    use crate::bindings;

    pub type DFSHandle = uint32_t;

    #[repr(i32)]
    #[derive(Clone, Copy)]
    pub enum DFSResult {
        Success = 0,
        BadInput = -1,
        NoFile = -2,
        BadFS = -3,
        NoMem = -4,
        BadHandle = -5
    }

    /// FILE = 0b0000
    /// DIR =  0b0001
    /// EOF =  0b0010
    #[repr(C)]
    pub union Flag_Error {
        pub error: DFSResult,
        pub flags: c_int
    }

    /// EOF = 1
    /// Not_EOF = 0
    #[repr(C)]
    pub union EOFResult {
        pub error: DFSResult,
        pub eof: c_int
    }

    #[repr(C)]
    pub union OpenResult {
        pub error: DFSResult,
        pub handle: DFSHandle
    }

    #[repr(C)]
    pub union ReadResult {
        pub error: DFSResult,
        pub num: c_int
    }

    #[repr(C)]
    pub union TellResult {
        pub error: DFSResult,
        pub offset: c_int
    }

    #[repr(C)]
    pub union SizeResult {
        pub error: DFSResult,
        pub size: c_int
    }

    pub const DFS_DEFAULT_LOCATION: u64 = 0xB0101000;
    pub const MAX_OPEN_FILES: u64 = 4;
    pub const MAX_FILENAME_LEN: u64 = 243;
    pub const MAX_DIRECTORY_DEPTH: u64 = 100;
    pub const FLAGS_FILE: u64 = 0x0;
    pub const FLAGS_DIR: u64 = 0x1;
    pub const FLAGS_EOF: u64 = 0x2;

    /// Macro to extract the file type from a DragonFS file flag.
    #[inline(always)] pub extern "C" fn FILETYPE(x: u64) -> u64 { return x & 3; }

    /// Initialize the filesystem.
    ///
    /// Given a base offset where the filesystem should be found, this function
    /// will initialize the filesystem to read from cartridge space. This function
    /// will also register DragonFS with newlib so that standard POSIX file
    /// operations work with DragonFS.
    pub fn init(base_fs_location: u32) -> DFSResult {
        unsafe {
            return match bindings::dfs_init(base_fs_location) {
                0 => DFSResult::Success,
                -1 => DFSResult::BadInput,
                -2 => DFSResult::NoFile,
                -3 => DFSResult::BadFS,
                -4 => DFSResult::NoMem,
                -5 => DFSResult::BadHandle,
                bad => panic!("Invalid result from DragonFS::init(): {}", bad)
            };
        }
    }

    /// Change directories to the specified path.
    ///
    /// Supports absolute and relative
    ///
    /// Note: path must be null-terminated.
    pub fn chdir(path: &str) -> DFSResult {
        let cstr: *const i8 = CString::new(path).expect("At DragonFS::chdir()").as_ptr();

        unsafe {
            return match bindings::dfs_chdir(cstr) {
                0 => DFSResult::Success,
                -1 => DFSResult::BadInput,
                -2 => DFSResult::NoFile,
                -3 => DFSResult::BadFS,
                -4 => DFSResult::NoMem,
                -5 => DFSResult::BadHandle,
                bad => panic!("Invalid result from DragonFS::chdir(): {}", bad)
            };
        }
    }

    /// Find the first file or directory in a directory listing.
    ///
    /// Supports absolute and relative. If the path is invalid, returns a negative
    /// DFSResult. If a file or directory is found, returns the flags of the entry
    /// and copies the name into buffer.
    ///
    /// Note: path must be null-terminated.
    pub fn dir_find_first(path: &str, buffer_out: &mut [c_char]) -> Flag_Error {
        let cstr: *const i8 = CString::new(path).expect("At DragonFS::dir_find_first()").as_ptr();

        unsafe {
            return match bindings::dfs_dir_findfirst(cstr, buffer_out.as_mut_ptr()) {
                x @ 0..=3 => Flag_Error { flags: x },
                -1 => Flag_Error{ error: DFSResult::BadInput },
                -2 => Flag_Error{ error: DFSResult::NoFile },
                -3 => Flag_Error{ error: DFSResult::BadFS },
                -4 => Flag_Error{ error: DFSResult::NoMem },
                -5 => Flag_Error{ error: DFSResult::BadHandle },
                bad => panic!("Invalid result from DragonFS::dir_find_first(): {}", bad)
            };
        }
    }

    /// Find the next file or directory in a directory listing.
    ///
    /// Note: Should be called after doing a dir_find_first().
    pub fn dir_find_next(buffer_out: &mut str) -> Flag_Error {
        unsafe {
            return match bindings::dfs_dir_findnext(buffer_out.as_mut_ptr().cast()) {
                x @ 0..=3 => Flag_Error { flags: x },
                -1 => Flag_Error{ error: DFSResult::BadInput },
                -2 => Flag_Error{ error: DFSResult::NoFile },
                -3 => Flag_Error{ error: DFSResult::BadFS },
                -4 => Flag_Error{ error: DFSResult::NoMem },
                -5 => Flag_Error{ error: DFSResult::BadHandle },
                bad => panic!("Invalid result from DragonFS::dir_find_next(): {}", bad)
            };
        }
    }

    /// Open a file given a path.
    ///
    /// Check if we have any free file handles, and if we do, try to open
    /// the file specified. Supports absolute and relative paths
    ///
    /// Note: path must be null-terminated.
    pub fn open(path: &str) -> OpenResult {
        let cstr: *const i8 = CString::new(path).expect("At DragonFS::open()").as_ptr();

        unsafe {
            return match bindings::dfs_open(cstr) {
                -1 => OpenResult{ error: DFSResult::BadInput},
                -2 => OpenResult{ error: DFSResult::NoFile},
                -3 => OpenResult{ error: DFSResult::BadFS},
                -4 => OpenResult{ error: DFSResult::NoMem},
                -5 => OpenResult{ error: DFSResult::BadHandle},
                x @ c_int::MIN..=-6 => panic!("Invalid result from DragonFS::open(): {}", x),
                val => OpenResult{ handle: val as u32}
            };
        }
    }

    /// Read data from a file.
    pub fn read(buffer_out: *const (), size: i32, count: i32, handle: DFSHandle) -> ReadResult {
        unsafe {
            return match bindings::dfs_read(buffer_out.cast(), size, count, handle) {
                x @ 0..=c_int::MAX => ReadResult{ num: x },
                -1 => ReadResult{ error: DFSResult::BadInput},
                -2 => ReadResult{ error: DFSResult::NoFile},
                -3 => ReadResult{ error: DFSResult::BadFS},
                -4 => ReadResult{ error: DFSResult::NoMem},
                -5 => ReadResult{ error: DFSResult::BadHandle},
                bad => panic!("Invalid result from DragonFS::read(): {}", bad)
            }
        }
    }

    /// Seek to an offset in the file.
    pub fn seek(handle: DFSHandle, offset: i32, origin: i32) -> DFSResult {
        unsafe {
            return match bindings::dfs_seek(handle, offset, origin) {
                0 => DFSResult::Success,
                -1 => DFSResult::BadInput,
                -2 => DFSResult::NoFile,
                -3 => DFSResult::BadFS,
                -4 => DFSResult::NoMem,
                -5 => DFSResult::BadHandle,
                bad => panic!("Invalid result from DragonFS::seek(): {}", bad)
            };
        }
    }

    /// Return the current offset into a file.
    pub fn tell(handle: DFSHandle) -> TellResult {
        unsafe {
            return match bindings::dfs_tell(handle) {
                -1 => TellResult{ error: DFSResult::BadInput},
                -2 => TellResult{ error: DFSResult::NoFile},
                -3 => TellResult{ error: DFSResult::BadFS},
                -4 => TellResult{ error: DFSResult::NoMem},
                -5 => TellResult{ error: DFSResult::BadHandle},
                x @ c_int::MIN..=-6 => panic!("Invalid result from DragonFS::tell(): {}", x),
                bad => TellResult{ offset: bad }
            };
        }
    }

    /// Close an already open file handle.
    pub fn close(handle: DFSHandle) -> DFSResult {
        unsafe {
            return match bindings::dfs_close(handle) {
                0 => DFSResult::Success,
                -1 => DFSResult::BadInput,
                -2 => DFSResult::NoFile,
                -3 => DFSResult::BadFS,
                -4 => DFSResult::NoMem,
                -5 => DFSResult::BadHandle,
                bad => panic!("Invalid result from DragonFS::close(): {}", bad)
            };
        }
    }

    /// Return whether the end of file has been reached.
    pub fn eof(handle: DFSHandle) -> EOFResult {
        unsafe {
            return match bindings::dfs_eof(handle) {
                1 => EOFResult{ eof: 1 },
                0 => EOFResult{ eof: 0 },
                -1 => EOFResult{ error: DFSResult::BadInput },
                -2 => EOFResult{ error: DFSResult::NoFile },
                -3 => EOFResult{ error: DFSResult::BadFS },
                -4 => EOFResult{ error: DFSResult::NoMem },
                -5 => EOFResult{ error: DFSResult::BadHandle },
                bad => panic!("Invalid result from DragonFS::eof(): {}", bad)
            };
        }
    }

    /// Return the size of an open file.
    pub fn size(handle: DFSHandle) -> SizeResult {
        unsafe {
            return match bindings::dfs_size(handle) {
                -1 => SizeResult{ error: DFSResult::BadInput},
                -2 => SizeResult{ error: DFSResult::NoFile},
                -3 => SizeResult{ error: DFSResult::BadFS},
                -4 => SizeResult{ error: DFSResult::NoMem},
                -5 => SizeResult{ error: DFSResult::BadHandle},
                x @ c_int::MIN..=-6 => panic!("Invalid result from DragonFS::size(): {}", x),
                bad => SizeResult{ size: bad }
            };
        }
    }
}

/// Software routines for manipulating graphics in a display context.
///
/// The graphics subsystem is responsible for software manipulation of a display
/// context as returned from the Display Subsystem. All of the functions use a
/// pure software drawing method and are thus much slower than hardware sprite
/// support. However, they are slightly more flexible and offer no hardware limitations
/// in terms of sprite size.
///
/// Code wishing to draw to the screen should first acquire a display contect using
/// Display::lock(). Once the display context is acquired, code may draw to the
/// context using any of the graphics functions present. Wherever practical, two
/// versions of graphics functions are available: a transparent variety and a
/// non-transparent variety. Code that wishes to display sprites without transparency
/// can get a slight performance boost by using the non-transparent variety of calls
/// since no software alpha blending needs to occur. Once code has finished drawing to
/// the display context, it can be displayed to the screen using Display::show().
///
/// The graphics subsystem makes use of the same contexts as the Hardware Display
/// Interface. Thus, with careful coding, both hardware and software routines can be
/// used to draw to the display context with no ill effects. The colors returned by
/// make_color() and convert_color() are also compatible with both hardware and software
/// graphics routines.
pub mod GraphicsEngine {
    use cstr_core::CString;
    use cty::*;

    use crate::{Display::DisplayContext, bindings};


    #[repr(C)]
    pub struct RGBColor {
        pub r: uint8_t,
        pub g: uint8_t,
        pub b: uint8_t,
        pub a: uint8_t
    }

    pub type N64Color = uint32_t;

    #[repr(C)]
    pub struct Sprite {
        pub width: uint16_t,
        pub height: uint16_t,
        pub bitdepth: uint8_t,
        pub format: uint8_t,
        pub hslices: uint8_t,
        pub vslices: uint8_t,
        pub data: [uint32_t; 0]
    }

    /// Return a 32-bit representation of an RGBA color.
    pub fn make_color(r: i32, g: i32, b: i32, a: i32) -> N64Color {
        unsafe { return bindings::graphics_make_color(r, g, b, a); }
    }

    /// Convert a color structure to a 32-bit representation of an RGBA color.
    pub fn convert_color(color: RGBColor) -> N64Color {
        unsafe { return bindings::graphics_convert_color(color); }
    }

    /// Draw a pixel to a given display context.
    pub fn draw_pixel(disp: DisplayContext, x: i32, y: i32, c: N64Color) {
        unsafe { bindings::graphics_draw_pixel(disp, x, y, c); }
    }

    /// Draw a pixel to a given display context with alpha support.
    pub fn draw_pixel_trans(disp: DisplayContext, x: i32, y: i32, c: N64Color) {
        unsafe { bindings::graphics_draw_pixel_trans(disp, x, y, c); }
    }

    /// Draw a line to a given display context.
    pub fn draw_line(disp: DisplayContext, x0: i32, y0: i32, x1: i32, y1: i32, c: N64Color) {
        unsafe { bindings::graphics_draw_line(disp, x0, y0, x1, y1, c); }
    }

    /// Draw a line to a given display context with alpha support.
    pub fn draw_line_trans(disp: DisplayContext, x0: i32, y0: i32, x1: i32, y1: i32, c: N64Color) {
        unsafe { bindings::graphics_draw_line_trans(disp, x0, y0, x1, y1, c); }
    }

    /// Draw a filled rectangle to a display context.
    pub fn draw_box(disp: DisplayContext, x: i32, y: i32, width: i32, height: i32, color: N64Color) {
        unsafe { bindings::graphics_draw_box(disp, x, y, width, height, color); }
    }

    /// Draw a filled rectangle to a display context.
    pub fn draw_box_trans(disp: DisplayContext, x: i32, y: i32, width: i32, height: i32, color: N64Color) {
        unsafe { bindings::graphics_draw_box_trans(disp, x, y, width, height, color); }
    }

    /// Fill the entire screen with a particular color.
    pub fn fill_screen(disp: DisplayContext, c: N64Color) {
        unsafe { bindings::graphics_fill_screen(disp, c); }
    }

    /// Set the current forecolor and backcolor for text operations.
    pub fn set_color(forecolor: N64Color, backcolor: N64Color) {
        unsafe { bindings::graphics_set_color(forecolor, backcolor); }
    }

    /// Draw a character from the built-in font to the screen. This function does
    /// not support alpha blending, only binary transparency. If the background
    /// color is fully transparent, the font is drawn with no background. Otherwise,
    /// the font is drawn on a fully colored background. The foreground and background
    /// can be set using set_color().
    pub fn draw_character(disp: DisplayContext, x: i32, y: i32, c: u8) {
        unsafe { bindings::graphics_draw_character(disp, x, y, c as c_char); }
    }

    /// Draw a null terminated string to a display context.
    ///
    /// Draw a string to the screen, following a few simple rules. Standard ASCII is
    /// supported, as well as \r, \n, space and tab. \r and \n will both cause the
    /// next character to be rendered one line lower and at the x coordinate specified
    /// in the parameters. The tab character inserts five spaces.
    ///
    /// This function does not support alpha blending, only binary transparency. If
    /// the background color is fully transparent, the font is drawn with no background.
    /// Otherwise, the font is drawn on a fully colored background. The foreground and
    /// background can be set using set_color().
    ///
    /// Note: msg must be null-terminated.
    pub fn draw_text(disp: DisplayContext, x: i32, y: i32, msg: &str) {
        let cstr: *const i8 = CString::new(msg).expect("At GraphicsEngine::draw_text()").as_ptr();

        unsafe {
            bindings::graphics_draw_text(disp, x, y, cstr);
        }
    }

    /// Draw a sprite to a display context.
    ///
    /// Given a sprite structure, this function will draw a sprite to the display
    /// context with clipping support.
    pub fn draw_sprite(disp: DisplayContext, x: i32, y: i32, sprite: &mut Sprite) {
        unsafe { bindings::graphics_draw_sprite(disp, x, y, sprite); }
    }

    /// Draw a sprite from a spritemap to a display context.
    ///
    /// Given a sprite structure, this function will draw a sprite out of a larger
    /// spritemap to the display context with clipping support. This function is
    /// useful for software tilemapping. If a sprite was generated as a spritemap
    /// (it has more than one horizontal or vertical slice), this function can
    /// display a slice of the sprite as a standalone sprite.
    pub fn draw_sprite_stride(disp: DisplayContext, x: i32, y: i32, sprite: &mut Sprite, offset: i32) {
        unsafe { bindings::graphics_draw_sprite_stride(disp, x, y, sprite, offset); }
    }

    /// Draw a sprite to a display context with alpha transparency.
    ///
    /// Given a sprite structure, this function will draw a sprite to the display
    /// context with clipping support.
    pub fn draw_sprite_trans(disp: DisplayContext, x: i32, y: i32, sprite: &mut Sprite) {
        unsafe { bindings::graphics_draw_sprite_trans(disp, x, y, sprite); }
    }

    /// Draw a sprite from a spritemap to a display context.
    ///
    /// Given a sprite structure, this function will draw a sprite out of a larger
    /// spritemap to the display context with clipping support. This function is useful
    /// for software tilemapping. If a sprite was generated as a spritemap (it has
    /// more than one horizontal or vertical slice), this function can display a slice
    /// of the sprite as a standalone sprite.
    pub fn draw_sprite_stride_trans(disp: DisplayContext, x: i32, y: i32, sprite: &mut Sprite, offset: i32) {
        unsafe { bindings::graphics_draw_sprite_trans_stride(disp, x, y, sprite, offset); }
    }
}

/// N64 interrupt registering and servicing routines.
///
/// The N64 interrupt controller provides a software interface to register for interrupts
/// from the various systems in the N64. Most interrupts on the N64 coordinate through
/// the MIPS interface (MI) to allow interrupts to be handled at one spot. A notable
/// exception is the timer interrupt which is generated by the MIPS r4300 itself and not
/// the N64 hardware.
///
/// Before interrupts can be used on the system, the interrupt controller should be
/// configured using init(). Once this is done, interrupts are enabled and any registered
/// callback can be called when an interrupt occurs. Each of the N64-generated interrupts
/// is maskable using the various set accessors.
///
/// Interrupts can be enabled or disabled as a whole on the N64 using enable_interrupts()
/// and disable_interrupts(). It is assumed that once the interrupt system is activated,
/// these will always be called in pairs. Calling enable_interrupts() without first calling
/// disable_interrupts() is considered a violation of this assumption and should be avoided.
/// Calling disable_interrupts() when interrupts are already disabled will have no effect.
/// Calling enable_interrupts() again to restore from a critical section will not enable
/// interrupts if interrupts were not enabled when calling disable_interrupts(). In this
/// manner, it is safe to nest calls to disable and enable interrupts.
pub mod Interrupt {
    use crate::bindings;

    #[repr(C)]
    pub enum InterruptState {
        INTERRUPTS_UNINITIALIZED,
        INTERRUPTS_DISABLED,
        INTERRUPTS_ENABLED
    }

    pub type InterruptFlag = bool;

    /// Register an AI callback.
    pub fn register_AI_handler(callback: &mut extern "C" fn()) {
        unsafe { bindings::register_AI_handler(callback); }
    }

    /// Register a VI callback.
    pub fn register_VI_handler(callback: &mut extern "C" fn()) {
        unsafe { bindings::register_VI_handler(callback); }
    }

    /// Register a PI callback.
    pub fn register_PI_handler(callback: &mut extern "C" fn()) {
        unsafe { bindings::register_PI_handler(callback); }
    }

    /// Register a DP callback.
    pub fn register_DP_handler(callback: &mut extern "C" fn()) {
        unsafe { bindings::register_DP_handler(callback); }
    }

    /// Register a TI callback.
    pub fn register_TI_handler(callback: &mut extern "C" fn()) {
        unsafe { bindings::register_TI_handler(callback); }
    }

    /// Register an SI callback.
    pub fn register_SI_handler(callback: &mut extern "C" fn()) {
        unsafe { bindings::register_SI_handler(callback); }
    }

    /// Register an SP callback.
    pub fn register_SP_handler(callback: &mut extern "C" fn()) {
        unsafe { bindings::register_SP_handler(callback); }
    }

    /// Unregister an AI callback.
    pub fn unregister_AI_handler(callback: &mut extern "C" fn()) {
        unsafe { bindings::unregister_AI_handler(callback); }
    }

    /// Unregister a VI callback.
    pub fn unregister_VI_handler(callback: &mut extern "C" fn()) {
        unsafe { bindings::unregister_VI_handler(callback); }
    }

    /// Unregister a PI callback.
    pub fn unregister_PI_handler(callback: &mut extern "C" fn()) {
        unsafe { bindings::unregister_PI_handler(callback); }
    }

    /// Unregister a DP callback.
    pub fn unregister_DP_handler(callback: &mut extern "C" fn()) {
        unsafe { bindings::unregister_DP_handler(callback); }
    }

    /// Unregister a TI callback.
    pub fn unregister_TI_handler(callback: &mut extern "C" fn()) {
        unsafe { bindings::unregister_TI_handler(callback); }
    }

    /// Unregister an SI callback.
    pub fn unregister_SI_handler(callback: &mut extern "C" fn()) {
        unsafe { bindings::unregister_SI_handler(callback); }
    }

    /// Unregister an SP callback.
    pub fn unregister_SP_handler(callback: &mut extern "C" fn()) {
        unsafe { bindings::unregister_SP_handler(callback); }
    }

    /// Enable or disable AI interrupt.
    pub fn set_AI_interrupt(active: InterruptFlag) {
        unsafe { bindings::set_AI_interrupt(active as i32); }
    }

    /// Enable or disable VI interrupt. line is the vertical line that triggers this interrupt.
    pub fn set_VI_interrupt(active: InterruptFlag, line: u32) {
        unsafe { bindings::set_VI_interrupt(active as i32, line); }
    }

    /// Enable or disable PI interrupt.
    pub fn set_PI_interrupt(active: InterruptFlag) {
        unsafe { bindings::set_PI_interrupt(active as i32); }
    }

    /// Enable or disable DP interrupt.
    pub fn set_DP_interrupt(active: InterruptFlag) {
        unsafe { bindings::set_DP_interrupt(active as i32); }
    }

    /// Enable or disable SI interrupt.
    pub fn set_SI_interrupt(active: InterruptFlag) {
        unsafe { bindings::set_SI_interrupt(active as i32); }
    }

    /// Enable or disable SP interrupt.
    pub fn set_SP_interrupt(active: InterruptFlag) {
        unsafe { bindings::set_SP_interrupt(active as i32); }
    }

    /// Initialize the interrupt controller.
    pub fn init() {
        unsafe { bindings::init_interrupts(); }
    }

    /// Enable interrupts systemwide.
    ///
    /// Note: If this is called inside a nested disable call, it will have no effect
    /// on the system. Therefore it is safe to nest disable/enable calls. After the
    /// last nested interrupt is enabled, systemwide interrupts will be reenabled.
    pub fn enable_interrupts() {
        unsafe { bindings::enable_interrupts(); }
    }

    /// Disable interrupts systemwide.
    ///
    /// Note: If interrupts are already disabled on the system or interrupts
    /// have not been initialized, this function will not modify the system state.
    pub fn disable_interrupts() {
        unsafe { bindings::disable_interrupts(); }
    }

    /// Return the current state of interrupts.
    pub fn get_interrupts_state() -> InterruptState {
        unsafe { return bindings::get_interrupts_state(); }
    }
}

/// N64 bootup and cache interfaces.
///
/// The N64 system interface provides a way for code to interact with the memory
/// setup on the system. This includes cache operations to invalidate or flush
/// regions and the ability to set the boot CIC. The newlib Interface Hooks use the
/// knowledge of the boot CIC to properly determine if the expansion pak is present,
/// giving 4MB of additional memory. Aside from this, the MIPS r4300 uses a manual
/// cache management strategy, where SW that requires passing buffers to and from
/// hardware components using DMA controllers needs to ensure that cache and RDRAM
/// are in sync. A set of operations to invalidate and/or write back cache is
/// provided for both instruction cache and data cache.
#[allow(dead_code)]
pub mod N64System {
    use volatile::Volatile;

    use crate::bindings;

    #[repr(C)]
    pub enum TVType {
        TV_PAL = 0,
        TV_NTSC = 1,
        TV_MPAL = 2
    }

    /// Number of updates to the count register per second.
    ///
    /// Every second, this many counts will have passed in the count register
    pub const TICKS_PER_SECOND: u64 = 93750000 / 2;

    /// Return the uncached memory address for a given address.
    #[inline(always)]
    pub fn get_uncached_address(addr: u32) -> *mut () {
        return bindings::UncachedAddr(addr).cast();
    }

    /// Return the uncached memory address for a given address.
    #[inline(always)]
    pub fn get_uncached_short_address(addr: u32) -> *mut i16 {
        return bindings::UncachedShortAddr(addr);
    }

    /// Return the uncached memory address for a given address.
    #[inline(always)]
    pub fn get_uncached_unsigned_short_address(addr: u32) -> *mut u16 {
        return bindings::UncachedUShortAddr(addr);
    }

    /// Return the uncached memory address for a given address.
    #[inline(always)]
    pub fn get_uncached_long_address(addr: u32) -> *mut i32 {
        return bindings::UncachedLongAddr(addr);
    }

    /// Return the uncached memory address for a given address.
    #[inline(always)]
    pub fn get_uncached_unsigned_long_address(addr: u32) -> *mut u32 {
        return bindings::UncachedULongAddr(addr);
    }

    /// Return the cached memory address for a given address.
    #[inline(always)]
    pub fn get_cached_address(addr: u32) -> *mut () {
        return bindings::CachedAddr(addr).cast();
    }

    /// Memory barrier to ensure in-order execution.
    #[inline(always)]
    pub fn MEMORY_BARRIER() {
        bindings::MEMORY_BARRIER();
    }

    /// Returns the COP0 register $9 (count).
    ///
    /// The coprocessor 0 (system control coprocessor - COP0) register $9 is
    /// incremented at "half maximum instruction issue rate" which is the
    /// processor clock speed (93.75MHz) divided by two. (also see TICKS_PER_SECOND)
    /// It will always produce a 32-bit unsigned value which overflows back to zero
    /// in 91.625 seconds. The counter will increment irrespective of instructions
    /// actually being executed. This macro is for reading that value. Do not use
    /// for comparison without special handling.
    #[inline(always)]
    pub fn get_ticks_read() -> u32 {
        unsafe { return *bindings::TICKS_READ() };
    }

    /// The signed difference of time between "from" and "to".
    ///
    /// If "from" is before "to", the distance in time is positive,
    /// otherwise it is negative.
    #[inline(always)]
    pub fn get_ticks_distance(from: u32, to: u32) -> i32 {
        return bindings::TICKS_DISTANCE(from, to);
    }

    /// Returns true if "t1" is before "t2".
    ///
    /// This is similar to t1 < t2, but it correctly handles timer overflows
    /// which are very frequent. Notice that the N64 counter overflows every ~91
    /// seconds, so it's not possible to compare times that are more than ~45
    /// seconds apart.
    #[inline(always)]
    pub fn get_ticks_before(t1: u32, t2: u32) -> bool {
        return bindings::TICKS_BEFORE(t1, t2);
    }

    ///
    #[inline(always)]
    pub fn get_ticks_from_ms(val: u32) -> u32 {
        return bindings::TICKS_FROM_MS(val);
    }

    ///
    #[inline(always)]
    pub fn get_ticks() -> Volatile<u32> {
        unsafe { return bindings::get_ticks() };
    }

    ///
    #[inline(always)]
    pub fn get_ticks_ms() -> Volatile<u32> {
        unsafe { return bindings::get_ticks_ms() };
    }

    /// Return the boot CIC.
    pub fn get_boot_cic() -> i32 {
        unsafe { return bindings::sys_get_boot_cic(); }
    }

    /// Set the boot CIC.
    ///
    /// This function will set the boot CIC. If the value isn't in the range
    /// of 6102-6106, the boot CIC is set to the default of 6102.
    pub fn set_boot_cic(bc: i32) {
        unsafe { bindings::sys_set_boot_cic(bc); }
    }

    /// Spin wait until the number of ticks have elapsed.
    pub fn wait_ticks(wait: u32) {
        unsafe { bindings::wait_ticks(wait); }
    }

    /// Spin wait until the number of millisecounds have elapsed.
    pub fn wait_ms(wait_ms: u32) {
        unsafe { bindings::wait_ms(wait_ms); }
    }

    /// Force a data cache invalidate over a memory region.
    ///
    /// Use this to force the N64 to update cache from RDRAM.
    pub fn data_cache_hit_invalidate(addr: *mut (), length: u32) {
        unsafe { bindings::data_cache_hit_invalidate(Volatile::new(addr.cast()), length); }
    }

    /// Force a data cache writeback over a memory region.
    ///
    /// Use this to force cached memory to be written to RDRAM.
    pub fn data_cache_hit_writeback(addr: *mut (), length: u32) {
        unsafe { bindings::data_cache_hit_writeback(Volatile::new(addr.cast()), length); }
    }

    /// Force a data cache writeback invalidate over a memory region.
    ///
    /// Use this to force cached memory to be written to RDRAM and then cache updated.
    pub fn data_cache_hit_writeback_invalidate(addr: *mut (), length: u32) {
        unsafe { bindings::data_cache_hit_writeback_invalidate(Volatile::new(addr.cast()), length); }
    }

    /// Force a data cache index writeback invalidate over a memory region.
    pub fn data_cache_index_writeback_invalidate(addr: *mut (), length: u32) {
        unsafe { bindings::data_cache_index_writeback_invalidate(Volatile::new(addr.cast()), length); }
    }

    /// Force an instruction cache writeback over a memory region.
    ///
    /// Use this to force cached memory to be written to RDRAM.
    pub fn inst_cache_hit_writeback(addr: *mut (), length: u32) {
        unsafe { bindings::inst_cache_hit_writeback(Volatile::new(addr.cast()), length); }
    }

    /// Force an instruction cache invalidate over a memory region.
    ///
    /// Use this to force the N64 to update cache from RDRAM.
    pub fn inst_cache_hit_invalidate(addr: *mut (), length: u32) {
        unsafe { bindings::inst_cache_hit_invalidate(Volatile::new(addr.cast()), length); }
    }

    /// Force an instruction cache index invalidate over a memory region.
    pub fn inst_cache_index_invalidate(addr: *mut (), length: u32) {
        unsafe { bindings::inst_cache_index_invalidate(Volatile::new(addr.cast()), length); }
    }

    /// Get amount of available memory.
    pub fn get_memory_size() -> i32 {
        unsafe { return bindings::get_memory_size(); }
    }

    /// Is expansion pak in use.
    ///
    /// Checks whether the maximum available memory has been expanded to 8MB
    pub fn is_memory_expanded() -> bool {
        unsafe { return bindings::is_memory_expanded(); }
    }

    /// Is system NTSC/PAL/MPAL.
    //
    // Checks enum hard-coded in PIF BootROM to indicate the tv type of the system.
    pub fn get_tv_type() -> TVType {
        unsafe { return bindings::get_tv_type(); }
    }
}

/// N64 COP0 Interface.
pub mod COP0 {
    use crate::bindings;

    pub const C0_STATUS_IE: u64 = 0x0000_0001;
    pub const C0_STATUS_EXL: u64 = 0x0000_0002;
    pub const C0_STATUS_ERL: u64 = 0x0000_0004;
    pub const C0_CAUSE_BD: u64 = 0x8000_0000;
    pub const C0_CAUSE_CE: u64 = 0x3000_0000;
    pub const C0_CAUSE_EXC_CODE: u64 = 0x0000_007C;
    pub const C0_INTERRUPT_0: u64 = 0x0000_0100;
    pub const C0_INTERRUPT_1: u64 = 0x0000_0200;
    pub const C0_INTERRUPT_RCP: u64 = 0x0000_0400;
    pub const C0_INTERRUPT_3: u64 = 0x0000_0800;
    pub const C0_INTERRUPT_4: u64 = 0x0000_1000;
    pub const C0_INTERRUPT_5: u64 = 0x0000_2000;
    pub const C0_INTERRUPT_6: u64 = 0x0000_4000;
    pub const C0_INTERRUPT_TIMER: u64 = 0x0000_8000;

    /// Read the COP0 Count register
    #[inline(always)]
    pub fn COUNT() -> u32 {
        unsafe { return *bindings::C0_COUNT() };
    }

    /// Write the COP0 Count register.
    #[inline(always)]
    pub fn WRITE_COUNT(x: u32) {
        bindings::C0_WRITE_COUNT(x);
    }

    /// Read the COP0 Compare register.
    #[inline(always)]
    pub fn COMPARE() -> u32 {
        return bindings::C0_COMPARE();
    }

    /// Write the COP0 Compare register.
    #[inline(always)]
    pub fn WRITE_COMPARE(x: u32) {
        bindings::C0_WRITE_COMPARE(x);
    }

    /// Read the COP0 Status register.
    #[inline(always)]
    pub fn STATUS() -> u32 {
        return bindings::C0_STATUS();
    }

    /// Write the COP0 Status register.
    #[inline(always)]
    pub fn WRITE_STATUS(x: u32) {
        bindings::C0_WRITE_STATUS(x);
    }

    /// Returns the COP0 register $13 (Cause Register)
    ///
    /// The coprocessor 0 (system control coprocessor - COP0) register $13 is a read write
    /// register keeping pending interrupts, exception code, coprocessor unit number
    /// referenced for a coprocessor unusable exception.
    #[inline(always)]
    pub fn READ_CR() -> u32 {
        return bindings::C0_READ_CR();
    }

    /// Write the COP0 register $13 (Cause register)
    ///
    /// Use this to update it for a custom exception handler.
    #[inline(always)]
    pub fn WRITE_CR(x: u32) {
        bindings::C0_WRITE_CR(x);
    }

    /// Returns the COP0 register $8 (BadVAddr)
    ///
    /// The coprocessor 0 (system control coprocessor - COP0) register $8 is a read only
    /// register holding the last virtual address to be translated which became invalid,
    /// or a virtual address for which an addressing error occurred.
    #[inline(always)]
    pub fn READ_BADVADDR() -> u32 {
        return bindings::C0_READ_BADVADDR();
    }

    /// Read the COP0 register $14 (EPC)
    ///
    /// The coprocessor 0 (system control coprocessor - COP0) register $14 is the return
    /// from exception program counter. For asynchronous exceptions it points to the place
    /// to continue execution whereas for synchronous (caused by code) exceptions, point
    /// to the instruction causing the fault condition, which needs correction in the
    /// exception handler. This macro is for reading its value.
    #[inline(always)]
    pub fn READ_EPC() -> u32 {
        return bindings::C0_READ_EPC();
    }

    /// Get the CE value from the COP0 status register.
    ///
    /// Gets the Coprocessor unit number referenced by a coprocessor unusable exception
    /// from the given COP0 Status register value.
    #[inline(always)]
    pub fn GET_CAUSE_CE(cr: u64) -> u64 {
        return bindings::C0_GET_CAUSE_CE(cr);
    }
}

pub mod COP1 {
    use crate::bindings;

    pub const C1_FLAG_INEXACT_OP: u64 = 0x0000_0004;
    pub const C1_FLAG_UNDERFLOW: u64 = 0x0000_0008;
    pub const C1_FLAG_OVERFLOW: u64 = 0x0000_0010;
    pub const C1_FLAG_DIV_BY_0: u64 = 0x0000_0020;
    pub const C1_FLAG_INVALID_OP: u64 = 0x0000_0040;
    pub const C1_ENABLE_INEXACT_OP: u64 = 0x0000_0080;
    pub const C1_ENABLE_UNDERFLOW: u64 = 0x0000_0100;
    pub const C1_ENABLE_OVERFLOW: u64 = 0x0000_0200;
    pub const C1_ENABLE_DIV_BY_0: u64 = 0x0000_0400;
    pub const C1_ENABLE_INVALID_OP: u64 = 0x0000_0800;
    pub const C1_CAUSE_INEXACT_OP: u64 = 0x0000_1000;
    pub const C1_CAUSE_UNDERFLOW: u64 = 0x0000_2000;
    pub const C1_CAUSE_OVERFLOW: u64 = 0x0000_4000;
    pub const C1_CAUSE_DIV_BY_0: u64 = 0x0000_8000;
    pub const C1_CAUSE_INVALID_OP: u64 = 0x0001_0000;
    pub const C1_CAUSE_NOT_IMPLEMENTED: u64 = 0x0002_0000;

    /// Read the COP1 FCR31 register (floating-point control register 31)
    ///
    /// FCR31 is also known as the Control/Status register.
    /// It keeps control and status data for the FPU.
    #[inline(always)]
    pub fn FCR31() -> u32 {
        return bindings::C1_FCR31();
    }

    /// Write to the COP1 FCR31 register.
    #[inline(always)]
    pub fn WRITE_FCR31(x: u32) {
        bindings::C1_WRITE_FCR31(x);
    }
}

/// Interface to the hardware sprite/triangle rasterizer (RDP).
///
/// The hardware display interface sets up and talks with the RDP in order to render
/// hardware sprites, triangles and rectangles. The RDP is a very low level rasterizer
/// and needs data in a very specific format. The hardware display interface handles
/// this by building commands to be sent to the RDP.
///
/// Before attempting to draw anything using the RDP, the hardware display interface
/// should be initialized with init(). After the RDP is no longer needed, be sure
/// to free all resources using close().
///
/// Code wishing to use the hardware rasterizer should first acquire a display
/// context using Display::lock(). Once a display context has been acquired, the RDP
/// can be attached to the display context with attach_display(). Once the display has
/// been attached, the RDP can be used to draw sprites, rectangles and
/// textured/untextured triangles to the display context. Note that some functions
/// require additional setup, so read the descriptions for each function before use.
/// After code has finished rendering hardware assisted graphics to the display context,
/// the RDP can be detached from the context using detach_display(). After calling
/// this function, it is safe to immediately display the rendered graphics to the screen
/// using Display::show(), or additional software graphics manipulation can take place
/// using functions from the 2D Graphics.
///
/// Careful use of the sync() operation is required for proper rasterization. Before
/// performing settings changes such as clipping changes or setting up texture or solid
/// fill modes, code should perform a SYNC_PIPE. A SYNC_PIPE should be performed again
/// before any new texture load. This is to ensure that the last texture operation is
/// completed before attempting to change texture memory. Careful execution of texture
/// operations can allow code to skip some sync operations. Be careful with excessive
/// sync operations as it can stall the pipeline and cause triangles/rectangles to be
/// drawn on the next display context instead of the current.
///
/// detach_display() will automatically perform a SYNC_FULL to ensure that everything
/// has been completed in the RDP. This call generates an interrupt when complete which
/// signals the main thread that it is safe to detach. Consequently, interrupts must
/// be enabled for proper operation. This also means that code should under normal
/// circumstances never use SYNC_FULL.
pub mod RDP {
    use crate::{Display::DisplayContext, GraphicsEngine::{N64Color, Sprite}, bindings};

    /// RDP sync operations.
    #[repr(C)]
    pub enum Sync {
        SYNC_FULL, // Wait for any operation to complete before causing a DP interrupt.
        SYNC_PIPE, // Sync the RDP pipeline.
        SYNC_LOAD, // Block until all texture load operations are complete.
        SYNC_TILE  // Block until all tile operations are complete.
    }

    /// Mirror settings for textures.
    #[repr(C)]
    pub enum Mirror {
        MIRROR_DISABLED, // Disable texture mirroring.
        MIRROR_X,        // Enable texture mirroring on x axis.
        MIRROR_Y,        // Enable texture mirroring on y axis.
        MIRROR_XY        // Enable texture mirroring on both x & y axis.
    }

    /// Caching strategy for loaded textures.
    #[repr(C)]
    pub enum Flush {
        FLUSH_STRATEGY_NONE,     // Textures are assumed to be pre-flushed.
        FLUSH_STRATEGY_AUTOMATIC // Cache will be flushed on all incoming textures.
    }

    /// Initialize the RDP system.
    pub fn init() {
        unsafe { bindings::rdp_init(); }
    }

    /// Attach the RDP to a display context.
    ///
    /// This function allows the RDP to operate on display contexts fetched with
    /// Display::lock(). This should be performed before any other operations to
    /// ensure that the RDP has a valid output buffer to operate on.
    pub fn attach_display(disp: DisplayContext) {
        unsafe { bindings::rdp_attach_display(disp); }
    }

    /// Detach the RDP from a display context.
    ///
    /// Note: This function requires interrupts to be enabled to operate properly.
    ///
    /// This function will ensure that all hardware operations have completed on an
    /// output buffer before detaching the display context. This should be performed
    /// before displaying the finished output using Display::show()
    pub fn detach_display() {
        unsafe { bindings::rdp_detach_display(); }
    }

    /// Perform a sync operation.
    ///
    /// Do not use excessive sync operations between commands as this can cause the
    /// RDP to stall. If the RDP stalls due to too many sync operations, graphics may
    /// not be displayed until the next render cycle, causing bizarre artifacts. The
    /// rule of thumb is to only add a sync operation if the data you need is not yet
    /// available in the pipeline.
    pub fn sync(sync: Sync) {
        unsafe { bindings::rdp_sync(sync); }
    }

    /// Set the hardware clipping boundary.
    pub fn set_clipping(top_left_x: u32, top_left_y: u32, bottom_right_x: u32, bottom_right_y: u32) {
        unsafe { bindings::rdp_set_clipping(top_left_x, top_left_y, bottom_right_x, bottom_right_y); }
    }

    /// Set the hardware clipping boundary to the entire screen.
    pub fn set_default_clipping() {
        unsafe { bindings::rdp_set_default_clipping(); }
    }

    /// Enable display of 2D filled (untextured) rectangles.
    ///
    /// This must be called before using draw_filled_rectangle().
    pub fn enable_primitive_fill() {
        unsafe { bindings::rdp_enable_primitive_fill(); }
    }

    /// Enable display of 2D filled (untextured) triangles.
    ///
    /// This must be called before using draw_filled_triangle().
    pub fn enable_blend_fill() {
        unsafe { bindings::rdp_enable_blend_fill(); }
    }

    /// Enable display of 2D sprites.
    ///
    /// This must be called before using draw_textured_rectangle_scaled(),
    /// draw_textured_rectangle(), draw_sprite() or draw_sprite_scaled().
    pub fn enable_texture_copy() {
        unsafe { bindings::rdp_enable_texture_copy(); }
    }

    /// Load a sprite into RDP TMEM.
    ///
    /// Returns: number of bytes consumed in RDP TMEM by loading this sprite
    pub fn load_texture(tex_slot: u32, tex_location: u32, mirror: Mirror, sprite: &mut Sprite) -> u32 {
        unsafe { return bindings::rdp_load_texture(tex_slot, tex_location, mirror, sprite); }
    }

    /// Load part of a sprite into RDP TMEM.
    ///
    /// Given a sprite with vertical and horizontal slices defined, this
    /// function will load the slice specified in offset into texture memory.
    /// This is useful for treating a large sprite as a tilemap.
    ///
    /// Returns: number of bytes consumed in RDP TMEM by loading this sprite
    pub fn load_texture_stride(tex_slot: u32, tex_location: u32, mirror: Mirror, sprite: &mut Sprite, offset: i32) -> u32 {
        unsafe { return bindings::rdp_load_texture_stride(tex_slot, tex_location, mirror, sprite, offset); }
    }

    /// Draw a textured rectangle.
    ///
    /// Given an already loaded texture, this function will draw a rectangle textured
    /// with the loaded texture. If the rectangle is larger than the texture, it will
    /// be tiled or mirrored based on the* mirror setting given in the load texture command.
    ///
    /// Before using this command to draw a textured rectangle, use enable_texture_copy()
    /// to set the RDP up in texture mode.
    pub fn draw_textured_rectangle(tex_slot: u32, top_left_x: i32, top_left_y: i32, bottom_right_x: i32, bottom_right_y: i32, mirror: Mirror) {
        unsafe { bindings::rdp_draw_textured_rectangle(tex_slot, top_left_x, top_left_y, bottom_right_x, bottom_right_y, mirror); }
    }

    /// Draw a textured rectangle with a scaled texture.
    ///
    /// Given an already loaded texture, this function will draw a rectangle textured
    /// with the loaded texture at a scale other than 1. This allows rectangles to be
    /// drawn with stretched or squashed textures. If the rectangle is larger than the
    /// texture after scaling, it will be tiled or mirrored based on the mirror setting
    /// given in the load texture command.
    ///
    /// Before using this command to draw a textured rectangle, use enable_texture_copy()
    /// to set the RDP up in texture mode.
    pub fn draw_textured_rectangle_scaled(tex_slot: u32, top_left_x: i32, top_left_y: i32, bottom_right_x: i32, bottom_right_y: i32, x_scale: f64, y_scale: f64, mirror: Mirror) {
        unsafe { bindings::rdp_draw_textured_rectangle_scaled(tex_slot, top_left_x, top_left_y, bottom_right_x, bottom_right_y, x_scale, y_scale, mirror); }
    }

    /// Draw a texture to the screen as a sprite.
    ///
    /// Given an already loaded texture, this function will draw a rectangle textured
    /// with the loaded texture.
    ///
    /// Before using this command to draw a textured rectangle, use enable_texture_copy()
    /// to set the RDP up in texture mode.
    pub fn draw_sprite(tex_slot: u32, top_left_x: i32, top_left_y: i32, mirror: Mirror) {
        unsafe { bindings::rdp_draw_sprite(tex_slot, top_left_x, top_left_y, mirror); }
    }

    /// Draw a texture to the screen as a scaled sprite.
    ///
    /// Given an already loaded texture, this function will draw a rectangle textured
    /// with the loaded texture.
    ///
    /// Before using this command to draw a textured rectangle, use enable_texture_copy()
    /// to set the RDP up in texture mode.
    pub fn draw_sprite_scaled(tex_slot: u32, top_left_x: i32, top_left_y: i32, x_scale: f64, y_scale: f64, mirror: Mirror) {
        unsafe { bindings::rdp_draw_sprite_scaled(tex_slot, top_left_x, top_left_y, x_scale, y_scale, mirror); }
    }

    /// Set the primitive draw color for subsequent filled primitive operations.
    ///
    /// This function sets the color of all rdp_draw_filled_rectangle operations that
    /// follow. Note that in 16 bpp mode, the color must be a packed color. This means
    /// that the high 16 bits and the low 16 bits must both be the same color. Use
    /// Graphics::make_color() or Graphics::convert_color() to generate valid colors.
    pub fn set_primitive_color(color: N64Color) {
        unsafe { bindings::rdp_set_primitive_color(color); }
    }

    /// Set the blend draw color for subsequent filled primitive operations.
    ///
    /// This function sets the color of all draw_filled_triangle() operations that follow.
    pub fn set_blend_color(color: N64Color) {
        unsafe { bindings::rdp_set_blend_color(color); }
    }

    /// Draw a filled rectangle.
    ///
    /// Given a color set with set_primitive_color(), this will draw a filled rectangle
    /// to the screen. This is most often useful for erasing a buffer before drawing to
    /// it by displaying a black rectangle the size of the screen. This is much faster
    /// than setting the buffer blank in software. However, if you are planning on drawing
    /// to the entire screen, blanking may be unnecessary.
    /// Before calling this function, make sure that the RDP is set to primitive mode by
    /// calling enable_primitive_fill().
    pub fn draw_filled_rectangle(top_left_x: i32, top_left_y: i32, bottom_right_x: i32, bottom_right_y: i32) {
        unsafe { bindings::rdp_draw_filled_rectangle(top_left_x, top_left_y, bottom_right_x, bottom_right_y); }
    }

    /// Draw a filled triangle.
    ///
    /// Given a color set with set_blend_color(), this will draw a filled triangle to the
    /// screen. Vertex order is not important.
    ///
    /// Before calling this function, make sure that the RDP is set to blend mode by
    /// calling enable_blend_fill().
    pub fn draw_filled_triangle(x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) {
        unsafe { bindings::rdp_draw_filled_triangle(x1, y1, x2, y2, x3, y3); }
    }

    /// Set the flush strategy for texture loads.
    ///
    /// If textures are guaranteed to be in uncached RDRAM or the cache is flushed before
    /// calling load operations, the RDP can be told to skip flushing the cache. This
    /// affords a good speedup. However, if you are changing textures in memory on the fly
    /// or otherwise do not want to deal with cache coherency, set the cache strategy to
    /// automatic to have the RDP flush cache before texture loads.
    pub fn set_texture_flush(flush: Flush) {
        unsafe { bindings::rdp_set_texture_flush(flush); }
    }

    /// Close the RDP system.
    ///
    /// This function closes out the RDP system and cleans up any internal memory allocated by init().
    pub fn close() {
        unsafe { bindings::rdp_close(); }
    }
}

/// Hardware Vector Interface.
pub mod RSP {
    use cty::*;

    use crate::bindings;

    ///
    pub fn init() {
        unsafe { bindings::rsp_init(); }
    }

    ///
    pub fn load_microcode(start: *mut (), size: c_ulong) {
        unsafe { bindings::load_ucode(start.cast(), size); }
    }

    ///
    pub fn read_microcode(start: *mut (), size: c_ulong) {
        unsafe { bindings::read_ucode(start.cast(), size); }
    }

    ///
    pub fn load_data(start: *mut (), size: c_ulong) {
        unsafe { bindings::load_data(start.cast(), size); }
    }

    ///
    pub fn read_data(start: *mut (), size: c_ulong) {
        unsafe { bindings::read_data(start.cast(), size); }
    }

    ///
    pub fn run_microcode() {
        unsafe { bindings::run_ucode(); }
    }
}

/// Interface to the timer module in the MIPS r4300 processor.
///
/// The timer subsystem allows code to receive a callback after a specified number
/// of ticks or microseconds. It interfaces with the MIPS coprocessor 0 to handle
/// the timer interrupt and provide useful timing services.
///
/// Before attempting to use the timer subsystem, code should call timer_init.
/// After the timer subsystem has been initialized, a new one-shot or continuous
/// timer can be created with new_timer(). To remove an expired one-shot timer or
/// a recurring timer, use delete_timer(). To temporarily stop a timer, use stop_timer().
/// To restart a stopped timer or an expired one-shot timer, use start_timer().
/// Once code no longer needs the timer subsystem, a call to timer_close() will free
/// all continuous timers and shut down the timer subsystem. Note that timers removed
/// with stop_timer() or expired one-short timers will not be removed automatically
/// and are the responsibility of the calling code to be freed, regardless of a call
/// to timer_close().
///
/// Because the MIPS internal counter wraps around after ~90 seconds (see TICKS_READ),
/// it's not possible to schedule a timer more than 90 seconds in the future.
pub mod Timer {
    use cty::*;

    use crate::bindings;

    #[repr(C)]
    pub struct TimerLink {
        pub left: c_uint,
        pub set: c_uint,
        pub ovfl: c_int,
        pub flags: c_int,
        pub callback: extern "C" fn(ovfl: c_int),
    }

    // Timer flags
    pub const TF_ONE_SHOT: c_ulong = 0;   // Timer should fire only once.
    pub const TF_CONTINUOUS: c_ulong = 1; // Timer should fire at a regular interval

    /// Calculate timer ticks based on microseconds.
    #[inline(always)]
    pub fn TIMER_TICKS(us: c_longlong) -> c_int {
        return bindings::TIMER_TICKS(us);
    }

    /// Calculate microseconds based on timer ticks.
    #[inline(always)]
    pub fn TIMER_MICROS(tk: c_longlong) -> c_int {
        return bindings::TIMER_MICROS(tk);
    }

    /// Calculate timer ticks based on microseconds.
    #[inline(always)]
    pub fn TIMER_TICKS_LL(us: c_longlong) -> c_longlong {
        return bindings::TIMER_TICKS_LL(us);
    }

    /// Calculate microseconds based on timer ticks.
    #[inline(always)]
    pub fn TIMER_MICROS_LL(tk: c_longlong) -> c_longlong {
        return bindings::TIMER_MICROS_LL(tk);
    }

    /// Initialize the timer subsystem.
    ///
    /// This function will reset the COP0 ticks counter to 0. Even if you later
    /// access the hardware counter directly (via TICKS_READ()), it should not
    /// be a problem if you call init() early in the application main.
    ///
    /// Do not modify the COP0 ticks counter after calling this function. Doing
    /// so will impede functionality of the timer module.
    pub fn init() {
        unsafe { bindings::timer_init(); }
    }

    /// Create a new timer and add to list.
    pub fn new_timer(ticks: c_int, flags: c_int, callback: extern "C" fn(overflow: c_int)) -> TimerLink {
        unsafe { return bindings::new_timer(ticks, flags, callback).read(); }
    }

    /// Start a timer not currently in the list.
    pub fn start_timer(timer: &mut TimerLink, ticks: c_int, flags: c_int, callback: extern "C" fn(overflow: c_int)) {
        unsafe { bindings::start_timer(timer, ticks, flags, callback); }
    }

    /// Stop a timer and remove it from the list.
    ///
    /// Note: This function does not free a timer structure. Use delete_timer() to do this.
    pub fn stop_timer(timer: &mut TimerLink) {
        unsafe { bindings::stop_timer(timer); }
    }

    /// Remove a timer from the list and delete it.
    pub fn delete_timer(timer: &mut TimerLink) {
        unsafe { bindings::delete_timer(timer); }
    }

    /// Free and close the timer subsystem.
    ///
    /// This function will ensure all recurring timers are deleted from the list before closing.
    /// One-shot timers that have expired will need to be manually deleted with delete_timer().
    pub fn timer_close() {
        unsafe { bindings::timer_close(); }
    }

    /// Return total ticks since timer was initialized, as a 64-bit counter.
    pub fn timer_ticks() -> c_long {
        unsafe { return bindings::timer_ticks(); }
    }
}

/// Handle hardware-generated exceptions.
///
/// The exception handler traps exceptions generated by hardware. This could be an invalid
/// instruction or invalid memory access exception or it could be a reset exception. In both cases,
/// a handler registered with register_exception_handler() will be passed information regarding
/// the exception type and relevant registers.
pub mod Exceptions {
    use cty::*;
    use volatile::Volatile;

    use crate::bindings;

    #[repr(C)]
    pub struct Exception {
        pub _type: c_int,
        pub code: ExceptionCode,
        pub info: *const c_char,
        pub regs: Volatile<RegisterBlock>
    }

    #[repr(C)]
    pub enum ExceptionType {
        EXCEPTION_TYPE_UNKNOWN = 0,
        EXCEPTION_TYPE_RESET,
        EXCEPTION_TYPE_CRITICAL
    }

    #[repr(C)]
    pub enum ExceptionCode {
        EXCEPTION_CODE_INTERRUPT = 0,
        EXCEPTION_CODE_TLB_MODIFICATION = 1,
        EXCEPTION_CODE_TLB_LOAD_I_MISS = 2,
        EXCEPTION_CODE_TLB_STORE_MISS = 3,
        EXCEPTION_CODE_LOAD_I_ADDRESS_ERROR = 4,
        EXCEPTION_CODE_STORE_ADDRESS_ERROR = 5,
        EXCEPTION_CODE_I_BUS_ERROR = 6,
        EXCEPTION_CODE_D_BUS_ERROR = 7,
        EXCEPTION_CODE_SYS_CALL = 8,
        EXCEPTION_CODE_BREAKPOINT = 9,
        EXCEPTION_CODE_RESERVED_INSTRUCTION = 10,
        EXCEPTION_CODE_COPROCESSOR_UNUSABLE = 11,
        EXCEPTION_CODE_ARITHMETIC_OVERFLOW = 12,
        EXCEPTION_CODE_TRAP = 13,
        EXCEPTION_CODE_FLOATING_POINT = 15,
        EXCEPTION_CODE_WATCH = 23,
    }

    #[repr(C)]
    pub struct RegisterBlock {
        pub gpr: [Volatile<c_ulong>; 32],
        pub sr: Volatile<c_uint>,
        pub cr: Volatile<c_uint>,
        pub epc: Volatile<c_uint>,
        pub hi: Volatile<c_ulong>,
        pub lo: Volatile<c_ulong>,
        pub fc31: Volatile<c_uint>,
        pub fpr: [Volatile<c_ulong>; 32]
    }

    /// Register an exception handler to handle exceptions.
    ///
    /// The registered handle is responsible for clearing any bits that may cause a re-trigger
    /// of the same exception and updating the EPC. An important example is the cause bits (12-17)
    /// of FCR31 from cop1. To prevent re-triggering the exception they should be cleared by the handler.
    ///
    /// To manipulate the registers, update the values in the Exception struct. They will be restored
    /// to appropriate locations when returning from the handler. Setting them directly will not work
    /// as expected as they will get overwritten with the values pointed by the struct.
    ///
    /// There is only one exception to this, cr (cause register) which is also modified by the int handler
    /// before the saved values are restored thus it is only possible to update it through COP0::WRITE_CR()
    /// macro if it is needed. This shouldn't be necessary though as they are already handled by the library.
    ///
    /// k0 ($26), k1 ($27) are not saved/restored and will not be available in the handler. Theoretically
    /// we can exclude s0-s7 ($16-$23), and gp ($28) to gain some performance as they are already saved
    /// by GCC when necessary. The same is true for sp ($29) and ra ($31) but current interrupt handler
    /// manipulates them via allocating a new stack and doing a jal. Similarly floating point registers
    /// f21-f31 are callee-saved. In the future we may consider removing them from the save state for
    /// interrupts (but not for exceptions)
    pub fn register_exception_handler(callback: extern "C" fn(*mut Exception)) {
        unsafe { bindings::register_exception_handler(callback); }
    }

    ///
    pub fn default_exception_handler(exception: *mut Exception) {
        unsafe { bindings::exception_default_handler(exception); }
    }
}

/// Directory handling.
pub mod Directory {
    use cty::*;

    use crate::bindings;

    #[repr(C)]
    pub struct DirType {
        pub d_name: [c_char; 256],
        pub d_type: c_int
    }

    pub const DT_REG: c_ulong = 1; // Regular file
    pub const DT_DIR: c_ulong = 2; // Directory

    /// Find the first file in a directory.
    ///
    /// Note: Path must be null-terminated.
    pub fn find_first(path: &[c_uchar], dir: &mut DirType) -> c_int {
        unsafe { return bindings::dir_findfirst(path.as_ptr().cast(), dir); }
    }

    /// Find the next file in a directory.
    ///
    /// Note: Path must be null-terminated.
    pub fn find_next(path: &[c_uchar], dir: &mut DirType) -> c_int {
        unsafe { return bindings::dir_findnext(path.as_ptr().cast(), dir); }
    }
}