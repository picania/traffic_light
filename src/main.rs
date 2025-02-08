#![no_std]
#![no_main]

use core::time::Duration;

use arduino_hal::port::{mode::Output, Pin, PinOps};
use panic_halt as _;

// Timers
const FLASH_DUR: Duration = Duration::from_millis(500);
const STOP_WAIT: Duration = Duration::from_secs(20);
const READY_WAIT: Duration = Duration::from_secs(3);
const GO_WAIT: Duration = Duration::from_secs(15);
const ATTENTION_WAIT: Duration = READY_WAIT;

struct Traffic<R, O, G> {
    red: Pin<Output, R>,
    orange: Pin<Output, O>,
    green: Pin<Output, G>,
    state: State,
}

impl<R, O, G> Traffic<R, O, G> 
where
    R: PinOps,
    O: PinOps,
    G: PinOps,
{
    fn run(&mut self) -> ! {
        self.red.set_low();
        self.orange.set_low();
        self.green.set_low();

        let mut flash_counter = 0;
        loop {
            match self.state {
                State::NoOp(duration) => {
                    self.orange.toggle();
                    arduino_hal::delay_ms(duration.as_millis() as u16);
                    flash_counter += 1;

                    if flash_counter == 10 {
                        flash_counter = 0;
                        self.state = State::Stop(STOP_WAIT);
                    }
                }
                State::Stop(duration) => {
                    self.red.set_high();
                    arduino_hal::delay_ms(duration.as_millis() as u16);

                    self.state = State::Ready(READY_WAIT);
                }
                State::Ready(duration) => {
                    self.red.set_high();
                    self.orange.set_high();
                    arduino_hal::delay_ms(duration.as_millis() as u16);

                    self.state = State::Go(GO_WAIT);
                }
                State::Go(duration) => {
                    self.red.set_low();
                    self.orange.set_low();
                    self.green.set_high();
                    arduino_hal::delay_ms(duration.as_millis() as u16);

                    self.state = State::GoEnd(FLASH_DUR);
                }
                State::GoEnd(duration) => {
                    self.green.toggle();
                    arduino_hal::delay_ms(duration.as_millis() as u16);
                    flash_counter += 1;

                    if flash_counter == 5 {
                        flash_counter = 0;
                        self.state = State::Attention(ATTENTION_WAIT);
                    }
                }
                State::Attention(duration) => {
                    self.orange.set_high();
                    arduino_hal::delay_ms(duration.as_millis() as u16);

                    self.state = State::Stop(STOP_WAIT);
                    self.orange.set_low();
                },
            }

        }
    }
}

enum State {
    NoOp(Duration),
    Stop(Duration),
    Ready(Duration),
    Go(Duration),
    GoEnd(Duration),
    Attention(Duration),
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    /*
     * For examples (and inspiration), head to
     *
     *     https://github.com/Rahix/avr-hal/tree/main/examples
     *
     * NOTE: Not all examples were ported to all boards!  There is a good chance though, that code
     * for a different board can be adapted for yours.  The Arduino Uno currently has the most
     * examples available.
     */

    let mut traffic = Traffic {
        red: pins.d12.into_output(),
        orange: pins.d11.into_output(),
        green: pins.d10.into_output(),
        state: State::NoOp(FLASH_DUR),
    };

    traffic.run();
}
