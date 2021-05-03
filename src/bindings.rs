#![allow(asm_sub_register)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::sync::atomic::{Ordering, compiler_fence};
use cty::{c_char, c_double, c_float, c_int, c_long, c_longlong, c_short, c_uchar, c_uint, c_ulong, c_ushort, c_void, int32_t, uint16_t, uint32_t, uint8_t};
use volatile::Volatile;

use crate::{Audio::fill_buffer_callback, Controller::{ControllerData, ControllerOriginData, N64Controller, GCController}, Directory::{DirType}, Display::{AntiAlias, BitDepth, DisplayContext, Gamma, Resolution}, Exceptions::{Exception, ExceptionCode, ExceptionType, RegisterBlock}, GraphicsEngine::{RGBColor, Sprite}, Interrupt::{InterruptState}, MemoryPak::{EntryStructure}, N64System::{TVType}, RDP::{Sync, Flush, Mirror}, Timer::{TimerLink}, TransferPak::{GBCSupportType, GBCTitle, GameboyCartridgeHeader, NewTitle, OldTitle}};

/*
    cop0.h defines
 */
#[inline(always)]
pub(crate) extern "C" fn C0_COUNT() -> *const uint32_t {
    let x: *mut uint32_t = core::ptr::null_mut();
    unsafe { asm!("mfc0 {0},$9", out(reg) (*x)); }
    x
}

#[inline(always)]
pub(crate) extern "C" fn C0_WRITE_COUNT(x: uint32_t) {
    unsafe { asm!("mtc0 {0},$9", in(reg) x); }
}

#[inline(always)]
pub(crate) extern "C" fn C0_COMPARE() -> uint32_t {
    let x: uint32_t;
    unsafe { asm!("mfc0 {0},$11", out(reg) x); }
    x
}

#[inline(always)]
pub(crate) extern "C" fn C0_WRITE_COMPARE(x: uint32_t) {
    unsafe { asm!("mtc0 {0},$11", in(reg) x); }
}

#[inline(always)]
pub(crate) extern "C" fn C0_STATUS() -> uint32_t {
    let x: uint32_t;
    unsafe { asm!("mfc0 {0},$12", out(reg) x); }
    x
}

#[inline(always)]
pub(crate) extern "C" fn C0_WRITE_STATUS(x: uint32_t) {
    unsafe { asm!("mtc0 {0},$12", in(reg) x); }
}

#[inline(always)]
pub(crate) extern "C" fn C0_READ_CR() -> uint32_t {
    let x: uint32_t;
    unsafe { asm!("mfc0 {0},$13", out(reg) x); }
    x
}

#[inline(always)]
pub(crate) extern "C" fn C0_WRITE_CR(x: uint32_t) {
    unsafe { asm!("mtc0 {0},$13", in(reg) x); }
}

#[inline(always)]
pub(crate) extern "C" fn C0_READ_BADVADDR() -> uint32_t {
    let x: uint32_t;
    unsafe { asm!("mfc0 {0},$8", out(reg) x); }
    x
}

#[inline(always)]
pub(crate) extern "C" fn C0_READ_EPC() -> uint32_t {
    let x: uint32_t;
    unsafe { asm!("mfc0 {0},$14", out(reg) x); }
    x
}
const C0_CAUSE_CE: u64 = 0x3000_0000;
const C0_CAUSE_EXC_CODE: u64 = 0x0000_007C;

#[inline(always)] pub(crate) extern "C" fn C0_GET_CAUSE_CE(cr: u64) -> u64 { return ((cr) & C0_CAUSE_CE) >> 28; }
#[inline(always)] pub(crate) extern "C" fn C0_GET_CAUSE_EXC_CODE(sr: u64) -> u64 { return ((sr) & C0_CAUSE_EXC_CODE) >> 2; }

/*
    cop1.h defines
 */
#[inline(always)]
pub(crate) extern "C" fn C1_FCR31() -> uint32_t {
    let x: uint32_t;
    unsafe { asm!("cfc1 {0},$f31", out(reg) x); }
    x
}

#[inline(always)]
pub(crate) extern "C" fn C1_WRITE_FCR31(x: uint32_t) {
    unsafe { asm!("ctc1 {0},$f31", in(reg) x); }
}

/*
    n64sys.h defines
 */
#[inline(always)] pub(crate) extern "C" fn UncachedAddr(_addr: c_ulong) -> *mut c_void { return (_addr | 0x20000000) as *mut c_void; }
#[inline(always)] pub(crate) extern "C" fn UncachedShortAddr(_addr: c_ulong) -> *mut c_short { return (_addr | 0x20000000) as *mut c_short; }
#[inline(always)] pub(crate) extern "C" fn UncachedUShortAddr(_addr: c_ulong) -> *mut c_ushort { return (_addr | 0x20000000) as *mut c_ushort; }
#[inline(always)] pub(crate) extern "C" fn UncachedLongAddr(_addr: c_ulong) -> *mut c_long { return (_addr | 0x20000000) as *mut c_long; }
#[inline(always)] pub(crate) extern "C" fn UncachedULongAddr(_addr: c_ulong) -> *mut c_ulong { return (_addr | 0x20000000) as *mut c_ulong; }
#[inline(always)] pub(crate) extern "C" fn CachedAddr(_addr: c_ulong) -> *mut c_void { return (_addr &!0x20000000) as *mut c_void; }

#[inline(always)]
pub(crate) extern "C" fn MEMORY_BARRIER() {
    compiler_fence(Ordering::Release);
    compiler_fence(Ordering::Acquire);
    compiler_fence(Ordering::AcqRel);
}

#[inline(always)] pub(crate) extern "C" fn TICKS_READ() -> *const uint32_t { return C0_COUNT(); }

const TICKS_PER_SECOND: uint32_t = 93750000 / 2;

#[inline(always)] pub(crate) extern "C" fn TICKS_DISTANCE(from: uint32_t, to: uint32_t) -> int32_t { return (to - from) as int32_t; }
#[inline(always)] pub(crate) extern "C" fn TICKS_BEFORE(t1: uint32_t, t2: uint32_t) -> bool { return TICKS_DISTANCE(t1, t2) > 0; }
#[inline(always)] pub(crate) extern "C" fn TICKS_FROM_MS(val: c_uint) -> uint32_t { return val * (TICKS_PER_SECOND / 1000); }
#[inline(always)] pub(crate) unsafe extern "C" fn get_ticks() -> Volatile<c_ulong> { return Volatile::new((*TICKS_READ()) as u32); }
#[inline(always)] pub(crate) unsafe extern "C" fn get_ticks_ms() -> Volatile<c_ulong> { return Volatile::new((*TICKS_READ() as u32) / (TICKS_PER_SECOND / 1000) as u32); }

/*
    timer.h defines
 */
#[inline(always)] pub(crate) extern "C" fn TIMER_TICKS(us: c_longlong) -> c_int { return (us * (46875 / 1000 as c_longlong)) as c_int; }
#[inline(always)] pub(crate) extern "C" fn TIMER_MICROS(tk: c_longlong) -> c_int { return (tk * (1000 / 46875 as c_longlong)) as c_int; }
#[inline(always)] pub(crate) extern "C" fn TIMER_TICKS_LL(us: c_longlong) -> c_longlong { return us * 46875 / 1000; }
#[inline(always)] pub(crate) extern "C" fn TIMER_MICROS_LL(tk: c_longlong) -> c_longlong { return tk * 1000 / 46875; }

/*
    C function interface
 */
#[link(name = "dragon")]
extern "C" {

    /*
        audio.h functions
     */

    // void audio_init(const int frequency, int numbuffers);
    pub(crate) fn audio_init(frequency: c_int, numbuffers: c_int);

    // void audio_set_buffer_callback(audio_fill_buffer_callback fill_buffer_callback);
    pub(crate) fn audio_set_buffer_callback(fill_buffer_callback: audio_fill_buffer_callback);

    // void audio_pause(bool pause);
    pub(crate) fn audio_pause(pause: bool);

    // void audio_write(const short * const buffer);
    pub(crate) fn audio_write(buffer: *const c_short);

    // volatile int audio_can_write();
    pub(crate) fn audio_can_write() -> Volatile<c_int>;

    // void audio_write_silence();
    pub(crate) fn audio_write_silence();

    // void audio_close();
    pub(crate) fn audio_close();

    // int audio_get_frequency();
    pub(crate) fn audio_get_frequency() -> c_int;

    // int audio_get_buffer_length();
    pub(crate) fn audio_get_buffer_length() -> c_int;

    /*
        console.h functions
     */
    // void console_init();
    pub(crate) fn console_init();

    // void console_close();
    pub(crate) fn console_close();

    // void console_set_render_mode(int mode);
    pub(crate) fn console_set_render_mode(mode: c_int);

    // void console_clear();
    pub(crate) fn console_clear();

    // void console_render();
    pub(crate) fn console_render();

    /*
        controller.h functions
     */
    // void controller_init();
    pub(crate) fn controller_init();

    // void controller_read( struct controller_data * data);
    pub(crate) fn controller_read(data: *mut controller_data);

    // void controller_read_gc( struct controller_data * data, const uint8_t rumble[4]);
    pub(crate) fn controller_read_gc(data: *mut controller_data, rumble: *const [uint8_t; 4]);

    // void controller_read_gc_origin( struct controller_origin_data * data);
    pub(crate) fn controller_read_gc_origin(data: *mut controller_origin_data);

    // int get_controllers_present();
    pub(crate) fn get_controllers_present() -> c_int;

    // int get_accessories_present(struct controller_data * data);
    pub(crate) fn get_accessories_present(data: *mut controller_data) -> c_int;

    // void controller_scan();
    pub(crate) fn controller_scan();

    // struct controller_data get_keys_down();
    pub(crate) fn get_keys_down() -> controller_data;

    // struct controller_data get_keys_up();
    pub(crate) fn get_keys_up() -> controller_data;

    // struct controller_data get_keys_held();
    pub(crate) fn get_keys_held() -> controller_data;

    // struct controller_data get_keys_pressed();
    pub(crate) fn get_keys_pressed() -> controller_data;

    // int get_dpad_direction( int controller );
    pub(crate) fn get_dpad_direction(controller: c_int) -> c_int;

    // int read_mempak_address( int controller, uint16_t address, uint8_t *data );
    pub(crate) fn read_mempak_address(controller: c_int, address: uint16_t, data: *mut uint8_t) -> c_int;

    // int write_mempak_address( int controller, uint16_t address, uint8_t *data );
    pub(crate) fn write_mempak_address(controller: c_int, address: uint16_t, data: *mut uint8_t) -> c_int;

    // int identify_accessory( int controller );
    pub(crate) fn identify_accessory(controller: c_int) -> c_int;

    // void rumble_start( int controller );
    pub(crate) fn rumble_start(controller: c_int);

    // void rumble_stop( int controller );
    pub(crate) fn rumble_stop(controller: c_int);

    // void execute_raw_command( int controller, int command, int bytesout, int bytesin, unsigned char *out, unsigned char *in );
    pub(crate) fn execute_raw_command(controller: c_int, command: c_int, bytesout: c_int, bytesin: c_int, out: *mut c_uchar, _in: *mut c_uchar);

    // int eeprom_present();
    pub(crate) fn eeprom_present() -> c_int;

    // void eeprom_read(int block, uint8_t * const buf);
    pub(crate) fn eeprom_read(block: c_int, buf: *const uint8_t);

    // void eeprom_write(int block, const uint8_t * const data);
    pub(crate) fn eeprom_write(block: c_int, data: *const uint8_t);

    /*
        mempak.h functions
     */
    // int read_mempak_sector( int controller, int sector, uint8_t *sector_data );
    pub(crate) fn read_mempak_sector(controller: c_int, sector: c_int, sector_data: *mut uint8_t) -> c_int;

    // int write_mempak_sector( int controller, int sector, uint8_t *sector_data );
    pub(crate) fn write_mempak_sector(controller: c_int, sector: c_int, sector_data: *mut uint8_t) -> c_int;

    // int validate_mempak( int controller );
    pub(crate) fn validate_mempak(controller: c_int) -> c_int;

    // int get_mempak_free_space( int controller );
    pub(crate) fn get_mempak_free_space(controller: c_int) -> c_int;

    // int get_mempak_entry( int controller, int entry, entry_structure_t *entry_data );
    pub(crate) fn get_mempak_entry(controller: c_int, entry: c_int, entry_data: *mut entry_structure_t) -> c_int;

    // int format_mempak( int controller );
    pub(crate) fn format_mempak(controller: c_int) -> c_int;

    // int read_mempak_entry_data( int controller, entry_structure_t *entry, uint8_t *data );
    pub(crate) fn read_mempak_entry_data(controller: c_int, entry: *mut entry_structure_t, data: *mut uint8_t) -> c_int;

    // int write_mempak_entry_data( int controller, entry_structure_t *entry, uint8_t *data );
    pub(crate) fn write_mempak_entry_data(controller: c_int, entry: *mut entry_structure_t, data: *mut uint8_t) -> c_int;

    // int delete_mempak_entry( int controller, entry_structure_t *entry );
    pub(crate) fn delete_mempak_entry(controller: c_int, entry: *mut entry_structure_t) -> c_int;

    /*
        tpak.h functions
     */
    // int tpak_init(int controller);
    pub(crate) fn tpak_init(controller: c_int) -> c_int;

    // int tpak_set_value(int controller, uint16_t address, uint8_t value);
    pub(crate) fn tpak_set_value(controller: c_int, address: uint16_t, value: uint8_t) -> c_int;

    // int tpak_set_bank(int controller, int bank);
    pub(crate) fn tpak_set_bank(controller: c_int, bank: c_int) -> c_int;

    // int tpak_set_power(int controller, bool power_state);
    pub(crate) fn tpak_set_power(controller: c_int, power_state: bool) -> c_int;

    // int tpak_set_access(int controller, bool access_state);
    pub(crate) fn tpak_set_access(controller: c_int, access_state: bool) -> c_int;

    // uint8_t tpak_get_status(int controller);
    pub(crate) fn tpak_get_status(controller: c_int) -> uint8_t;

    // int tpak_get_cartridge_header(int controller, struct gameboy_cartridge_header* header);
    pub(crate) fn tpak_get_cartridge_header(controller: c_int, header: *mut gameboy_cartridge_header) -> c_int;

    // bool tpak_check_header(struct gameboy_cartridge_header* header);
    pub(crate) fn tpak_check_header(header: *mut gameboy_cartridge_header) -> bool;

    // int tpak_write(int controller, uint16_t address, uint8_t* data, uint16_t size);
    pub(crate) fn tpak_write(controller: c_int, address: uint16_t, data: *mut uint8_t, size: uint16_t) -> c_int;

    // int tpak_read(int controller, uint16_t address, uint8_t* buffer, uint16_t size);
    pub(crate) fn tpak_read(controller: c_int, address: uint16_t, buffer: *mut uint8_t, size: uint16_t) -> c_int;

    /*
        display.h functions
     */
    // void display_init( resolution_t res, bitdepth_t bit, uint32_t num_buffers, gamma_t gamma, antialias_t aa );
    pub(crate) fn display_init(res: resolution_t, bit: bitdepth_t, num_buffers: uint32_t, gamma: gamma_t, aa: antialias_t);

    // display_context_t display_lock();
    pub(crate) fn display_lock() -> display_context_t;

    // void display_show(display_context_t disp);
    pub(crate) fn display_show(disp: display_context_t);

    // void display_close();
    pub(crate) fn display_close();

    /*
        dma.h functions
     */
    // void dma_write(void * ram_address, unsigned long pi_address, unsigned long len);
    pub(crate) fn dma_write(ram_address: *mut c_void, pi_address: c_ulong, len: c_ulong);

    // void dma_read(void * ram_address, unsigned long pi_address, unsigned long len);
    pub(crate) fn dma_read(ram_address: *mut c_void, pi_address: c_ulong, len: c_ulong);

    // volatile int dma_busy();
    pub(crate) fn dma_busy() -> Volatile<c_int>;

    // uint32_t io_read(uint32_t pi_address);
    pub(crate) fn io_read(pi_address: uint32_t) -> uint32_t;

    // void io_write(uint32_t pi_address, uint32_t data);
    pub(crate) fn io_write(pi_address: uint32_t, data: uint32_t);

    /*
        dragonfs.h functions
     */
    // int dfs_init(uint32_t base_fs_loc);
    pub(crate) fn dfs_init(base_fs_loc: uint32_t) -> c_int;

    // int dfs_chdir(const char * const path);
    pub(crate) fn dfs_chdir(path: *const c_char) -> c_int;

    // int dfs_dir_findfirst(const char * const path, char *buf);
    pub(crate) fn dfs_dir_findfirst(path: *const c_char, buf: *mut c_char) -> c_int;

    // int dfs_dir_findnext(char *buf);
    pub(crate) fn dfs_dir_findnext(buf: *mut c_char) -> c_int;

    // int dfs_open(const char * const path);
    pub(crate) fn dfs_open(path: *const c_char) -> c_int;

    // int dfs_read(void * const buf, int size, int count, uint32_t handle);
    pub(crate) fn dfs_read(buf: *const c_void, size: c_int, count: c_int, handle: uint32_t) -> c_int;

    // int dfs_seek(uint32_t handle, int offset, int origin);
    pub(crate) fn dfs_seek(handle: uint32_t, offset: c_int, origin: c_int) -> c_int;

    // int dfs_tell(uint32_t handle);
    pub(crate) fn dfs_tell(handle: uint32_t) -> c_int;

    // int dfs_close(uint32_t handle);
    pub(crate) fn dfs_close(handle: uint32_t) -> c_int;

    // int dfs_eof(uint32_t handle);
    pub(crate) fn dfs_eof(handle: uint32_t) -> c_int;

    // int dfs_size(uint32_t handle);
    pub(crate) fn dfs_size(handle: uint32_t) -> c_int;

    /*
        graphics.h functions
     */
    // uint32_t graphics_make_color( int r, int g, int b, int a );
    pub(crate) fn graphics_make_color(r: c_int, g: c_int, b: c_int, a: c_int) -> uint32_t;

    // uint32_t graphics_convert_color( color_t color );
    pub(crate) fn graphics_convert_color(color: color_t) -> uint32_t;

    // void graphics_draw_pixel( display_context_t disp, int x, int y, uint32_t c );
    pub(crate) fn graphics_draw_pixel(disp: display_context_t, x: c_int, y: c_int, c: uint32_t);

    // void graphics_draw_pixel_trans( display_context_t disp, int x, int y, uint32_t c );
    pub(crate) fn graphics_draw_pixel_trans(disp: display_context_t, x: c_int, y: c_int, c: uint32_t);

    // void graphics_draw_line( display_context_t disp, int x0, int y0, int x1, int y1, uint32_t c );
    pub(crate) fn graphics_draw_line(disp: display_context_t, x0: c_int, y0: c_int, x1: c_int, y1: c_int, c: uint32_t);

    // void graphics_draw_line_trans( display_context_t disp, int x0, int y0, int x1, int y1, uint32_t c );
    pub(crate) fn graphics_draw_line_trans(disp: display_context_t, x0: c_int, y0: c_int, x1: c_int, y1: c_int, c: uint32_t);

    // void graphics_draw_box( display_context_t disp, int x, int y, int width, int height, uint32_t color );
    pub(crate) fn graphics_draw_box(disp: display_context_t, x: c_int, y: c_int, width: c_int, height: c_int, color: uint32_t);

    // void graphics_draw_box_trans( display_context_t disp, int x, int y, int width, int height, uint32_t color );
    pub(crate) fn graphics_draw_box_trans(disp: display_context_t, x: c_int, y: c_int, width: c_int, height: c_int, color: uint32_t);

    // void graphics_fill_screen( display_context_t disp, uint32_t c );
    pub(crate) fn graphics_fill_screen(disp: display_context_t, c: uint32_t);

    // void graphics_set_color( uint32_t forecolor, uint32_t backcolor );
    pub(crate) fn graphics_set_color(forecolor: uint32_t, backcolor: uint32_t);

    // void graphics_draw_character( display_context_t disp, int x, int y, char c );
    pub(crate) fn graphics_draw_character(disp: display_context_t, x: c_int, y: c_int, c: c_char);

    // void graphics_draw_text( display_context_t disp, int x, int y, const char * const msg );
    pub(crate) fn graphics_draw_text(disp: display_context_t, x: c_int, y: c_int, msg: *const c_char);

    // void graphics_draw_sprite( display_context_t disp, int x, int y, sprite_t *sprite );
    pub(crate) fn graphics_draw_sprite(disp: display_context_t, x: c_int, y: c_int, sprite: *mut sprite_t);

    // void graphics_draw_sprite_stride( display_context_t disp, int x, int y, sprite_t *sprite, int offset );
    pub(crate) fn graphics_draw_sprite_stride(disp: display_context_t, x: c_int, y: c_int, sprite: *mut sprite_t, offset: c_int);

    // void graphics_draw_sprite_trans( display_context_t disp, int x, int y, sprite_t *sprite );
    pub(crate) fn graphics_draw_sprite_trans(disp: display_context_t, x: c_int, y: c_int, sprite: *mut sprite_t);

    // void graphics_draw_sprite_trans_stride( display_context_t disp, int x, int y, sprite_t *sprite, int offset );
    pub(crate) fn graphics_draw_sprite_trans_stride(disp: display_context_t, x: c_int, y: c_int, sprite: *mut sprite_t, offset: c_int);

    /*
        interrupt.h functions
     */
    // void register_AI_handler( void (*callback)() );
    pub(crate) fn register_AI_handler(callback: *mut extern "C" fn());

    // void register_VI_handler( void (*callback)() );
    pub(crate) fn register_VI_handler(callback: *mut extern "C" fn());

    // void register_PI_handler( void (*callback)() );
    pub(crate) fn register_PI_handler(callback: *mut extern "C" fn());

    // void register_DP_handler( void (*callback)() );
    pub(crate) fn register_DP_handler(callback: *mut extern "C" fn());

    // void register_TI_handler( void (*callback)() );
    pub(crate) fn register_TI_handler(callback: *mut extern "C" fn());

    // void register_SI_handler( void (*callback)() );
    pub(crate) fn register_SI_handler(callback: *mut extern "C" fn());

    // void register_SP_handler( void (*callback)() );
    pub(crate) fn register_SP_handler(callback: *mut extern "C" fn());

    // void unregister_AI_handler( void (*callback)() );
    pub(crate) fn unregister_AI_handler(callback: *mut extern "C" fn());

    // void unregister_VI_handler( void (*callback)() );
    pub(crate) fn unregister_VI_handler(callback: *mut extern "C" fn());

    // void unregister_PI_handler( void (*callback)() );
    pub(crate) fn unregister_PI_handler(callback: *mut extern "C" fn());

    // void unregister_DP_handler( void (*callback)() );
    pub(crate) fn unregister_DP_handler(callback: *mut extern "C" fn());

    // void unregister_TI_handler( void (*callback)() );
    pub(crate) fn unregister_TI_handler(callback: *mut extern "C" fn());

    // void unregister_SI_handler( void (*callback)() );
    pub(crate) fn unregister_SI_handler(callback: *mut extern "C" fn());

    // void unregister_SP_handler( void (*callback)() );
    pub(crate) fn unregister_SP_handler(callback: *mut extern "C" fn());

    // void set_AI_interrupt( int active );
    pub(crate) fn set_AI_interrupt(active: c_int);

    // void set_VI_interrupt( int active, unsigned long line );
    pub(crate) fn set_VI_interrupt(active: c_int, line: c_ulong);

    // void set_PI_interrupt( int active );
    pub(crate) fn set_PI_interrupt(active: c_int);

    // void set_DP_interrupt( int active );
    pub(crate) fn set_DP_interrupt(active: c_int);

    // void set_SI_interrupt( int active );
    pub(crate) fn set_SI_interrupt(active: c_int);

    // void set_SP_interrupt( int active );
    pub(crate) fn set_SP_interrupt(active: c_int);

    // void init_interrupts();
    pub(crate) fn init_interrupts();

    // void enable_interrupts();
    pub(crate) fn enable_interrupts();

    // void disable_interrupts();
    pub(crate) fn disable_interrupts();

    // interrupt_state_t get_interrupts_state();
    pub(crate) fn get_interrupts_state() -> interrupt_state_t;

    /*
        n64sys.h functions
     */
    // int sys_get_boot_cic();
    pub(crate) fn sys_get_boot_cic() -> c_int;

    // void sys_set_boot_cic(int bc);
    pub(crate) fn sys_set_boot_cic(bc: c_int);

    // void wait_ticks( unsigned long wait );
    pub(crate) fn wait_ticks(wait: c_ulong);

    // void wait_ms( unsigned long wait_ms );
    pub(crate) fn wait_ms(wait_ms: c_ulong);

    // void data_cache_hit_invalidate(volatile void *, unsigned long);
    pub(crate) fn data_cache_hit_invalidate(_: Volatile<*mut c_void>, _: c_ulong);

    // void data_cache_hit_writeback(volatile void *, unsigned long);
    pub(crate) fn data_cache_hit_writeback(_: Volatile<*mut c_void>, _: c_ulong);

    // void data_cache_hit_writeback_invalidate(volatile void *, unsigned long);
    pub(crate) fn data_cache_hit_writeback_invalidate(_: Volatile<*mut c_void>, _: c_ulong);

    // void data_cache_index_writeback_invalidate(volatile void *, unsigned long);
    pub(crate) fn data_cache_index_writeback_invalidate(_: Volatile<*mut c_void>, _: c_ulong);

    // void inst_cache_hit_writeback(volatile void *, unsigned long);
    pub(crate) fn inst_cache_hit_writeback(_: Volatile<*mut c_void>, _: c_ulong);

    // void inst_cache_hit_invalidate(volatile void *, unsigned long);
    pub(crate) fn inst_cache_hit_invalidate(_: Volatile<*mut c_void>, _: c_ulong);

    // void inst_cache_index_invalidate(volatile void *, unsigned long);
    pub(crate) fn inst_cache_index_invalidate(_: Volatile<*mut c_void>, _: c_ulong);

    // int get_memory_size();
    pub(crate) fn get_memory_size() -> c_int;

    // bool is_memory_expanded();
    pub(crate) fn is_memory_expanded() -> bool;

    // tv_type_t get_tv_type();
    pub(crate) fn get_tv_type() -> tv_type_t;

    /*
        rdp.h functions
     */
    // void rdp_init( void );
    pub(crate) fn rdp_init();

    // void rdp_attach_display( display_context_t disp );
    pub(crate) fn rdp_attach_display(disp: display_context_t);

    // void rdp_detach_display( void );
    pub(crate) fn rdp_detach_display();

    // void rdp_sync( sync_t sync );
    pub(crate) fn rdp_sync(sync: sync_t);

    // void rdp_set_clipping( uint32_t tx, uint32_t ty, uint32_t bx, uint32_t by );
    pub(crate) fn rdp_set_clipping(tx: uint32_t, ty: uint32_t, bx: uint32_t, by: uint32_t);

    // void rdp_set_default_clipping( void );
    pub(crate) fn rdp_set_default_clipping();

    // void rdp_enable_primitive_fill( void );
    pub(crate) fn rdp_enable_primitive_fill();

    // void rdp_enable_blend_fill( void );
    pub(crate) fn rdp_enable_blend_fill();

    // void rdp_enable_texture_copy( void );
    pub(crate) fn rdp_enable_texture_copy();

    // uint32_t rdp_load_texture( uint32_t texslot, uint32_t texloc, mirror_t mirror, sprite_t *sprite );
    pub(crate) fn rdp_load_texture(textslot: uint32_t, texloc: uint32_t, mirror: mirror_t, sprite: *mut sprite_t) -> uint32_t;

    // uint32_t rdp_load_texture_stride( uint32_t texslot, uint32_t texloc, mirror_t mirror, sprite_t *sprite, int offset );
    pub(crate) fn rdp_load_texture_stride(texslot: uint32_t, texloc: uint32_t, mirror: mirror_t, sprite: *mut sprite_t, offset: c_int) -> uint32_t;

    // void rdp_draw_textured_rectangle( uint32_t texslot, int tx, int ty, int bx, int by,  mirror_t mirror );
    pub(crate) fn rdp_draw_textured_rectangle(textslot: uint32_t, tx: c_int, ty: c_int, bx: c_int, by: c_int, mirror: mirror_t);

    // void rdp_draw_textured_rectangle_scaled( uint32_t texslot, int tx, int ty, int bx, int by, double x_scale, double y_scale,  mirror_t mirror );
    pub(crate) fn rdp_draw_textured_rectangle_scaled(texslot: uint32_t, tx: c_int, ty: c_int, bx: c_int, by: c_int, x_scale: c_double, y_scale: c_double, mirror: mirror_t);

    // void rdp_draw_sprite( uint32_t texslot, int x, int y ,  mirror_t mirror);
    pub(crate) fn rdp_draw_sprite(textslot: uint32_t, x: c_int, y: c_int, mirror: mirror_t);

    // void rdp_draw_sprite_scaled( uint32_t texslot, int x, int y, double x_scale, double y_scale,  mirror_t mirror);
    pub(crate) fn rdp_draw_sprite_scaled(textslot: uint32_t, x: c_int, y: c_int, x_scale: c_double, y_scale: c_double, mirror: mirror_t);

    // void rdp_set_primitive_color( uint32_t color );
    pub(crate) fn rdp_set_primitive_color(color: uint32_t);

    // void rdp_set_blend_color( uint32_t color );
    pub(crate) fn rdp_set_blend_color(color: uint32_t);

    // void rdp_draw_filled_rectangle( int tx, int ty, int bx, int by );
    pub(crate) fn rdp_draw_filled_rectangle(tx: c_int, ty: c_int, bx: c_int, by: c_int);

    // void rdp_draw_filled_triangle( float x1, float y1, float x2, float y2, float x3, float y3 );
    pub(crate) fn rdp_draw_filled_triangle(x1: c_float, y1: c_float, x2: c_float, y2: c_float, x3: c_float, y3: c_float);

    // void rdp_set_texture_flush( flush_t flush );
    pub(crate) fn rdp_set_texture_flush(flush: flush_t);

    // void rdp_close( void );
    pub(crate) fn rdp_close();

    /*
        rsp.h functions
     */
    // void rsp_init();
    pub(crate) fn rsp_init();

    // void load_ucode(void * start, unsigned long size);
    pub(crate) fn load_ucode(start: *mut c_void, size: c_ulong);

    // void read_ucode(void* start, unsigned long size);
    pub(crate) fn read_ucode(start: *mut c_void, size: c_ulong);

    // void load_data(void * start, unsigned long size);
    pub(crate) fn load_data(start: *mut c_void, size: c_ulong);

    // void read_data(void* start, unsigned long size);
    pub(crate) fn read_data(start: *mut c_void, size: c_ulong);

    // void run_ucode();
    pub(crate) fn run_ucode();

    /*
        timer.h functions
     */
    // void timer_init(void);
    pub(crate) fn timer_init();

    // timer_link_t *new_timer(int ticks, int flags, void (*callback)(int ovfl));
    pub(crate) fn new_timer(ticks: c_int, flags: c_int, callback: extern "C" fn(ovfl: c_int)) -> *mut timer_link_t;

    // void start_timer(timer_link_t *timer, int ticks, int flags, void (*callback)(int ovfl));
    pub(crate) fn start_timer(timer: *mut timer_link_t, ticks: c_int, flags: c_int, callback: extern "C" fn(ovfl: c_int));

    // void stop_timer(timer_link_t *timer);
    pub(crate) fn stop_timer(timer: *mut timer_link_t);

    // void delete_timer(timer_link_t *timer);
    pub(crate) fn delete_timer(timer: *mut timer_link_t);

    // void timer_close(void);
    pub(crate) fn timer_close();

    // long long timer_ticks(void);
    pub(crate) fn timer_ticks() -> c_long;

    /*
        exception.h functions
     */
    // void register_exception_handler( void (*cb)(exception_t *) );
    pub(crate) fn register_exception_handler(cb: extern "C" fn(*mut exception_t));

    // void exception_default_handler( exception_t* ex );
    pub(crate) fn exception_default_handler(ex: *mut exception_t);

    /*
        dir.h functions
     */
    // int dir_findfirst( const char * const path, dir_t *dir );
    pub(crate) fn dir_findfirst(path: *const c_char, dir: *mut dir_t) -> c_int;

    // int dir_findnext( const char * const path, dir_t *dir );
    pub(crate) fn dir_findnext(path: *const c_char, dir: *mut dir_t) -> c_int;
}

/*
    audio.h types
 */
type audio_fill_buffer_callback = fill_buffer_callback;

/*
    controller.h types
 */
type controller_data = _controller_data;
type _controller_data = ControllerData;
type _SI_condat = N64Controller;
type _SI_condat_gc = GCController;
type controller_origin_data = _controller_origin_data;
type _controller_origin_data = ControllerOriginData;

/*
    mempak.h types
 */
type entry_structure = EntryStructure;
type entry_structure_t = entry_structure;

/*
    tpak.h types
 */
type gameboy_cartridge_header = GameboyCartridgeHeader;
type gch_union = GBCTitle;
type old_gbc_title = OldTitle;
type new_gbc_title = NewTitle;
type gbc_support_type = GBCSupportType;

/*
    display.h types
 */
type resolution_t = Resolution;
type bitdepth_t = BitDepth;
type gamma_t = Gamma;
type antialias_t = AntiAlias;
type display_context_t = DisplayContext;

/*
    graphics.h types
 */
type color_t = RGBColor;
type sprite_t = Sprite;

/*
    interrupt.h types
 */
type interrupt_state_t = InterruptState;

/*
    n64sys.h types
 */
type tv_type_t = TVType;

/*
    rdp.h types
 */
type sync_t = Sync;
type mirror_t = Mirror;
type flush_t = Flush;

/*
    timer.h types
 */
type timer_link_t = TimerLink;

/*
    exception.h types
 */
type exception_t = Exception;
type unnamed_1 = ExceptionType;
type exception_code_t = ExceptionCode;
type reg_block_t = RegisterBlock;

/*
    dir.h types
 */
type dir_t = DirType;