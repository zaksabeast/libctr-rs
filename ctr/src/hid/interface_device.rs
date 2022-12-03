use super::PressedButtons;
use crate::svc;

/// An input device that supports the 3ds Buttons.
pub trait InterfaceDevice {
    /// Updates the previous and current inputs.
    /// Previous inputs are used for comparison to determine things like held buttons.
    /// This must be used to update the results of all other methods.
    fn scan_input();

    /// Gets the latest button u32.
    fn get_io_bits() -> u32;

    /// Gets the previous button u32.
    fn get_previous_io_bits() -> u32;

    /// Loops until no buttons are being pushed.
    fn wait_key_up() {
        loop {
            Self::scan_input();

            if Self::down_buttons().none() {
                break;
            }

            svc::sleep_thread(100000000);
        }
    }

    /// Gets the buttons currently being pressed,
    /// regardless of whether they were just pressed or actively being held.
    fn down_buttons() -> PressedButtons {
        PressedButtons::new(Self::get_io_bits())
    }

    /// Gets buttons that have been held down since the last scan.
    fn held_buttons() -> PressedButtons {
        PressedButtons::new(Self::get_io_bits() & Self::get_previous_io_bits())
    }

    // Gets the buttons that were pushed during the current scan.
    fn just_down_buttons() -> PressedButtons {
        PressedButtons::new(!Self::get_previous_io_bits() & Self::get_io_bits())
    }

    /// Checks if the button combination was just pressed.
    /// Primarily used for button combinations and the `Button` enum.
    ///
    /// Humans usually don't press two buttons at the exact same time.  Under the hood,
    /// this checks if any of the buttons were just pressed, and if all of the buttons are
    /// currently pressed.  This allows for a check to see if a button combination in its
    /// entirety was just pressed.
    fn is_just_pressed(io_bits: impl Into<u32>) -> bool {
        let io_bits = io_bits.into();
        ((Self::just_down_buttons() & io_bits) != 0) && io_bits == Self::down_buttons()
    }
}
