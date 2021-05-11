#![no_main]
#![no_std]

use core::panic::PanicInfo;

use libdragon_bindings::*;

#[no_mangle]
extern "C" fn _start () -> ! {
    Audio::init(500, 1);
    Console::init();
    Controller::init();
    Display::init(
        Display::Resolution::RESOLUTION_256x240,
        Display::BitDepth::DEPTH_16_BPP,
        1,
        Display::Gamma::GAMMA_NONE,
        Display::AntiAlias::ANTIALIAS_OFF
    );
    Interrupt::init();
    RDP::init();
    RSP::init();
    Timer::init();

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}