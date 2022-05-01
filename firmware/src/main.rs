//! RKB1 keyboard firmware
#![feature(allocator_api)]
#![feature(strict_provenance)]
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::*;
use defmt_rtt as _;
use embedded_time::fixed_point::FixedPoint;
use panic_probe as _;

use pio_proc::pio_file;
use rp_pico as bsp;

use bsp::hal::{
    clocks::init_clocks_and_plls,
    gpio::{FunctionPio0, Pins},
    pac,
    pio::PIOExt,
    sio::Sio,
    watchdog::Watchdog,
    Clock,
};

use crate::{allocator::HeapRoot, matrix_scan::MatrixScanner};

//extern crate alloc;

pub mod allocator;
pub mod matrix_scan;
pub mod sync;

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let sio = Sio::new(pac.SIO);
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    info!("init pin");
    let pins = Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    info!("pins");
    let _pins = (
        pins.gpio0.into_mode::<FunctionPio0>(),
        pins.gpio1.into_mode::<FunctionPio0>(),
        pins.gpio2.into_mode::<FunctionPio0>(),
        pins.gpio3.into_mode::<FunctionPio0>(),
        pins.gpio4.into_mode::<FunctionPio0>(),
        pins.gpio5.into_mode::<FunctionPio0>(),
        pins.gpio6.into_mode::<FunctionPio0>(),
        pins.gpio7.into_mode::<FunctionPio0>(),
        pins.gpio8.into_mode::<FunctionPio0>(),
        pins.gpio9.into_mode::<FunctionPio0>(),
        pins.gpio10.into_mode::<FunctionPio0>(),
    );
    info!("setup pins");

    let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);
    let mut matrix_scanner = MatrixScanner::init_pio_program(&mut pio, sm0);
    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().integer());

    loop {
        info!("starting matrix poll");
        info!("matrix poll result: 0x{:x}", matrix_scanner.poll());
        delay.delay_ms(1000);
    }
}
