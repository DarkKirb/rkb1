//! Matrix scanning routines

use defmt::{debug, info};
use pio::{Program, RP2040_MAX_PROGRAM_SIZE};
use pio_proc::pio_file;
use rp_pico::{
    hal::pio::{
        PIOBuilder, PinDir, Running, Rx, ShiftDirection, StateMachine, Tx, UninitStateMachine, PIO,
        PIO0SM0,
    },
    pac::{pio0::SM, Peripherals, PIO0},
};

/// Scanning input for the matrix
///
/// input is split into 6 6 bit fields and 28 bits of padding
///
/// each 6 bit field is a power of two, in order:
/// 1, 2, 4, 8, 16, 32
#[link_section = ".data"]
static MATRIX_SCAN_INPUT: [u32; 2] = [0x0408_0120, 0x4081_0000];

pub struct MatrixScanner {
    sm: StateMachine<PIO0SM0, Running>,
    tx: Tx<PIO0SM0>,
    rx: Rx<PIO0SM0>,
}

impl MatrixScanner {
    // Loads the program
    fn load_program() -> Program<RP2040_MAX_PROGRAM_SIZE> {
        pio_file!("src/matrix.pio").program
    }
    pub fn init_pio_program(pio: &mut PIO<PIO0>, sm0: UninitStateMachine<PIO0SM0>) -> Self {
        let program = Self::load_program();
        let installed = pio.install(&program).unwrap();
        let (mut sm, rx, tx) = PIOBuilder::from_program(installed)
            .out_shift_direction(ShiftDirection::Right)
            .out_pins(5, 6)
            .in_pin_base(0)
            .clock_divisor(0.0)
            .out_sticky(true)
            .build(sm0);
        sm.set_pindirs([
            (0, PinDir::Input),
            (1, PinDir::Input),
            (2, PinDir::Input),
            (3, PinDir::Input),
            (4, PinDir::Input),
            (5, PinDir::Output),
            (6, PinDir::Output),
            (7, PinDir::Output),
            (8, PinDir::Output),
            (9, PinDir::Output),
            (10, PinDir::Output),
        ]);

        Self {
            sm: sm.start(),
            tx,
            rx,
        }
    }
    pub fn poll(&mut self) -> u32 {
        self.tx.write(MATRIX_SCAN_INPUT);

        loop {
            unsafe {
                debug!("PADOUT: {:x}", (0x5020003c as *const u32).read_volatile());
                debug!("PADOE: {:x}", (0x50200040 as *const u32).read_volatile());
            }
            match self.rx.read() {
                Some(v) => return v,
                None => {}
            }
        }
    }
}
