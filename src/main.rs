//! Simple CAN example.
//! Requires a transceiver connected to PA11, PA12 (CAN1) or PB5 PB6 (CAN2).

#![feature(let_chains)]
#![no_main]
#![no_std]

use bxcan::{Fifo, Frame, Id, StandardId};
use panic_halt as _;

use bxcan::filter::Mask32;
use cortex_m_rt::entry;
use nb::block;
use stm32f1xx_hal::{can::Can, pac, prelude::*};
use stm32f1xx_hal::gpio::PinState;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();

    // To meet CAN clock accuracy requirements an external crystal or ceramic
    // resonator must be used. The blue pill has a 8MHz external crystal.
    // Other boards might have a crystal with another frequency or none at all.
    rcc.cfgr.use_hse(8.MHz()).freeze(&mut flash.acr);

    let mut afio = dp.AFIO.constrain();
    let mut gpioa = dp.GPIOA.split();
    let mut gpiob = dp.GPIOB.split();
    let mut gpioc = dp.GPIOC.split();

    let (pa15, pb3, _) = afio.mapr.disable_jtag(gpioa.pa15, gpiob.pb3, gpiob.pb4);

    let mut can_txr_gnd = pa15.into_push_pull_output(&mut gpioa.crh);
    can_txr_gnd.set_state(PinState::Low);

    let mut can_txr_vcc = pb3.into_push_pull_output(&mut gpiob.crl);
    can_txr_vcc.set_state(PinState::High);

    let mut can1 = {
        #[cfg(not(feature = "connectivity"))]
            let can = Can::new(dp.CAN1, dp.USB);
        #[cfg(feature = "connectivity")]
            let can = Can::new(dp.CAN1);

        let rx = gpioa.pa11.into_floating_input(&mut gpioa.crh);
        let tx = gpioa.pa12.into_alternate_push_pull(&mut gpioa.crh);
        can.assign_pins((tx, rx), &mut afio.mapr);

        // APB1 (PCLK1): 8MHz, Bit rate: 125kBit/s, Sample Point 87.5%
        // Value was calculated with http://www.bittiming.can-wiki.info/
        bxcan::Can::builder(can)
            .set_bit_timing(0x001c_0003)
            .leave_disabled()
    };

    // Configure filters so that can frames can be received.
    let mut filters = can1.modify_filters();
    filters.enable_bank(0, Fifo::Fifo0, Mask32::accept_all());

    let mut led_pin = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    let mut play_pin = gpioa.pa4.into_push_pull_output(&mut gpioa.crl);
    let mut next_pin = gpioa.pa3.into_push_pull_output(&mut gpioa.crl);
    let mut prev_pin = gpioa.pa5.into_push_pull_output(&mut gpioa.crl);

    led_pin.set_state(PinState::High);
    play_pin.set_state(PinState::High);
    next_pin.set_state(PinState::High);
    prev_pin.set_state(PinState::High);

    #[cfg(feature = "connectivity")]
        let _can2 = {
        let can = Can::new(dp.CAN2);

        let mut gpiob = dp.GPIOB.split();
        let rx = gpiob.pb5.into_floating_input(&mut gpiob.crl);
        let tx = gpiob.pb6.into_alternate_push_pull(&mut gpiob.crl);
        can.assign_pins((tx, rx), &mut afio.mapr);

        // APB1 (PCLK1): 8MHz, Bit rate: 125kBit/s, Sample Point 87.5%
        // Value was calculated with http://www.bittiming.can-wiki.info/
        let can2 = bxcan::Can::builder(can)
            .set_bit_timing(0x001c_0003)
            .leave_disabled();

        // A total of 28 filters are shared between the two CAN instances.
        // Split them equally between CAN1 and CAN2.
        let mut slave_filters = filters.set_split(14).slave_filters();
        slave_filters.enable_bank(14, Fifo::Fifo0, Mask32::accept_all());
        can2
    };

    // Drop filters to leave filter configuraiton mode.
    drop(filters);

    // Select the interface.
    let mut can = can1;
    //let mut can = _can2;

    // Split the peripheral into transmitter and receiver parts.
    block!(can.enable_non_blocking()).unwrap();

    // Echo back received packages in sequence.
    // See the `can-rtfm` example for an echo implementation that adheres to
    // correct frame ordering based on the transfer id.
    // rprintln!("start loop");
    let iframe = Frame::new_data(StandardId::new(1).unwrap(), [0x1, 0x2, 0x3, 0x4]);
    block!(can.transmit(&iframe)).unwrap();

    let mut mode_pressed = false;
    let mut seek_down_pressed = false;
    let mut seek_up_pressed = false;
    let mut sticky = false;

    loop {
        if let Ok(frame) = block!(can.receive()) {
            if let Id::Standard(id) = frame.id() && id.as_raw() == 0x2d5 && let Some(data) = frame.data() {
                mode_pressed = data[1] & 0x10 > 0;
                seek_down_pressed = data[1] & 0x80 > 0;
                seek_up_pressed = data[1] & 0x40 > 0;
            }
        }

        let seek_any_pressed = seek_up_pressed || seek_down_pressed;

        if !seek_any_pressed {
            sticky = false;
        }

        if mode_pressed && seek_any_pressed {
            sticky = true;
        }

        let play_state = sticky || mode_pressed;
        let p_next_state = !sticky && seek_up_pressed;
        let p_prev_state = !sticky && seek_down_pressed;

        led_pin.set_state(if play_state { PinState::High } else { PinState::Low });
        play_pin.set_state(if play_state { PinState::High } else { PinState::Low });
        next_pin.set_state(if p_next_state { PinState::High } else { PinState::Low });
        prev_pin.set_state(if p_prev_state { PinState::High } else { PinState::Low });
    }
}