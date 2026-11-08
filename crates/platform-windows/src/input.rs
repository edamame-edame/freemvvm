use freemvvm_core::abi_support::{DeviceScale, PlatformWindowInputEvent};
use std::sync::Arc;

const MODIFIER_SHIFT: u32 = 0x01;
const MODIFIER_CONTROL: u32 = 0x02;
const MODIFIER_ALT: u32 = 0x04;
const MODIFIER_META: u32 = 0x08;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PointerDevice {
    Mouse,
    Touch,
    Pen,
    Other,
}

pub(crate) fn modifier_mask(shift: bool, control: bool, alt: bool, meta: bool) -> u32 {
    (if shift { MODIFIER_SHIFT } else { 0 })
        | (if control { MODIFIER_CONTROL } else { 0 })
        | (if alt { MODIFIER_ALT } else { 0 })
        | (if meta { MODIFIER_META } else { 0 })
}

pub(crate) fn map_key_message(
    key_down: bool,
    lparam: isize,
    modifiers: u32,
) -> PlatformWindowInputEvent {
    let bits = lparam as u64;
    let scan_code = ((bits >> 16) & 0xff) as u8;
    let extended = ((bits >> 24) & 1) != 0;
    let identity = scan1_hid_usage(scan_code, extended).unwrap_or(0);
    if key_down {
        PlatformWindowInputEvent::KeyDown {
            identity,
            modifiers,
        }
    } else {
        PlatformWindowInputEvent::KeyUp {
            identity,
            modifiers,
        }
    }
}

pub(crate) fn scan1_hid_usage(scan_code: u8, extended: bool) -> Option<u32> {
    if extended {
        return Some(match scan_code {
            0x1c => 0x58, // Keypad Enter
            0x1d => 0xe4, // Right Control
            0x35 => 0x54, // Keypad /
            0x37 => 0x46, // Print Screen
            0x38 => 0xe6, // Right Alt
            0x47 => 0x4a, // Home
            0x48 => 0x52, // Up Arrow
            0x49 => 0x4b, // Page Up
            0x4b => 0x50, // Left Arrow
            0x4d => 0x4f, // Right Arrow
            0x4f => 0x4d, // End
            0x50 => 0x51, // Down Arrow
            0x51 => 0x4e, // Page Down
            0x52 => 0x49, // Insert
            0x53 => 0x4c, // Delete
            0x5b => 0xe3, // Left GUI
            0x5c => 0xe7, // Right GUI
            0x5d => 0x65, // Application
            _ => return None,
        });
    }

    Some(match scan_code {
        0x01 => 0x29,                                      // Escape
        0x02..=0x0a => 0x1e + u32::from(scan_code - 0x02), // 1 through 9
        0x0b => 0x27,                                      // 0
        0x0c => 0x2d,                                      // -
        0x0d => 0x2e,                                      // =
        0x0e => 0x2a,                                      // Backspace
        0x0f => 0x2b,                                      // Tab
        0x10 => 0x14,                                      // Q
        0x11 => 0x1a,                                      // W
        0x12 => 0x08,                                      // E
        0x13 => 0x15,                                      // R
        0x14 => 0x17,                                      // T
        0x15 => 0x1c,                                      // Y
        0x16 => 0x18,                                      // U
        0x17 => 0x0c,                                      // I
        0x18 => 0x12,                                      // O
        0x19 => 0x13,                                      // P
        0x1a => 0x2f,                                      // [
        0x1b => 0x30,                                      // ]
        0x1c => 0x28,                                      // Enter
        0x1d => 0xe0,                                      // Left Control
        0x1e => 0x04,                                      // A
        0x1f => 0x16,                                      // S
        0x20 => 0x07,                                      // D
        0x21 => 0x09,                                      // F
        0x22 => 0x0a,                                      // G
        0x23 => 0x0b,                                      // H
        0x24 => 0x0d,                                      // J
        0x25 => 0x0e,                                      // K
        0x26 => 0x0f,                                      // L
        0x27 => 0x33,                                      // ;
        0x28 => 0x34,                                      // '
        0x29 => 0x35,                                      // `
        0x2a => 0xe1,                                      // Left Shift
        0x2b => 0x31,                                      // backslash
        0x2c => 0x1d,                                      // Z
        0x2d => 0x1b,                                      // X
        0x2e => 0x06,                                      // C
        0x2f => 0x19,                                      // V
        0x30 => 0x05,                                      // B
        0x31 => 0x11,                                      // N
        0x32 => 0x10,                                      // M
        0x33 => 0x36,                                      // ,
        0x34 => 0x37,                                      // .
        0x35 => 0x38,                                      // /
        0x36 => 0xe5,                                      // Right Shift
        0x37 => 0x55,                                      // Keypad *
        0x38 => 0xe2,                                      // Left Alt
        0x39 => 0x2c,                                      // Space
        0x3a => 0x39,                                      // Caps Lock
        0x3b..=0x44 => 0x3a + u32::from(scan_code - 0x3b), // F1 through F10
        0x45 => 0x53,                                      // Num Lock
        0x46 => 0x47,                                      // Scroll Lock
        0x47 => 0x5f,                                      // Keypad 7
        0x48 => 0x60,                                      // Keypad 8
        0x49 => 0x61,                                      // Keypad 9
        0x4a => 0x56,                                      // Keypad -
        0x4b => 0x5c,                                      // Keypad 4
        0x4c => 0x5d,                                      // Keypad 5
        0x4d => 0x5e,                                      // Keypad 6
        0x4e => 0x57,                                      // Keypad +
        0x4f => 0x59,                                      // Keypad 1
        0x50 => 0x5a,                                      // Keypad 2
        0x51 => 0x5b,                                      // Keypad 3
        0x52 => 0x62,                                      // Keypad 0
        0x53 => 0x63,                                      // Keypad .
        0x57 => 0x44,                                      // F11
        0x58 => 0x45,                                      // F12
        0x56 => 0x64,                                      // Non-US backslash
        _ => return None,
    })
}

pub(crate) fn map_utf16_char(
    code_unit: u16,
    pending_high_surrogate: &mut Option<u16>,
) -> Option<PlatformWindowInputEvent> {
    let scalar = match code_unit {
        0xd800..=0xdbff => {
            *pending_high_surrogate = Some(code_unit);
            return None;
        }
        0xdc00..=0xdfff => {
            let high = pending_high_surrogate.take()?;
            0x1_0000 + ((u32::from(high) - 0xd800) << 10) + (u32::from(code_unit) - 0xdc00)
        }
        _ => {
            *pending_high_surrogate = None;
            u32::from(code_unit)
        }
    };
    let scalar = char::from_u32(scalar)?;
    Some(PlatformWindowInputEvent::TextCommit(Arc::from(
        scalar.to_string(),
    )))
}

pub(crate) fn map_pointer_button(
    key_down: bool,
    device: PointerDevice,
    primary: bool,
    first_button: bool,
    client_px: (i32, i32),
    scale: DeviceScale,
) -> Option<PlatformWindowInputEvent> {
    let supported_device = match device {
        PointerDevice::Mouse => true,
        PointerDevice::Touch | PointerDevice::Pen => primary && first_button,
        PointerDevice::Other => false,
    };
    if !supported_device {
        return None;
    }
    let x_dip = f64::from(client_px.0) / scale.x();
    let y_dip = f64::from(client_px.1) / scale.y();
    if !x_dip.is_finite() || !y_dip.is_finite() {
        return None;
    }
    Some(if key_down {
        PlatformWindowInputEvent::PrimaryPointerDown { x_dip, y_dip }
    } else {
        PlatformWindowInputEvent::PrimaryPointerUp { x_dip, y_dip }
    })
}

pub(crate) fn signed_client_pixels(lparam: isize) -> (i32, i32) {
    let bits = lparam as u64;
    let x = (bits as u16 as i16) as i32;
    let y = ((bits >> 16) as u16 as i16) as i32;
    (x, y)
}

#[cfg(test)]
mod tests {
    use super::{
        DeviceScale, PointerDevice, map_key_message, map_pointer_button, map_utf16_char,
        modifier_mask, scan1_hid_usage, signed_client_pixels,
    };
    use freemvvm_core::abi_support::PlatformWindowInputEvent as Event;
    use std::sync::Arc;

    fn key_lparam(scan_code: u8, extended: bool) -> isize {
        ((u64::from(scan_code) << 16) | (u64::from(extended) << 24)) as isize
    }

    #[test]
    fn platform_windows_input_maps_scan1_and_extended_hid_usages() {
        assert_eq!(scan1_hid_usage(0x1e, false), Some(0x04));
        assert_eq!(scan1_hid_usage(0x1d, false), Some(0xe0));
        assert_eq!(scan1_hid_usage(0x1d, true), Some(0xe4));
        assert_eq!(scan1_hid_usage(0x38, false), Some(0xe2));
        assert_eq!(scan1_hid_usage(0x38, true), Some(0xe6));
        assert_eq!(scan1_hid_usage(0x48, true), Some(0x52));
        assert_eq!(scan1_hid_usage(0x7f, false), None);
        assert_eq!(
            map_key_message(true, key_lparam(0x1e, false), 0x03),
            Event::KeyDown {
                identity: 0x04,
                modifiers: 0x03,
            }
        );
        assert_eq!(
            map_key_message(false, key_lparam(0x7f, false), 0),
            Event::KeyUp {
                identity: 0,
                modifiers: 0,
            }
        );
        assert_eq!(modifier_mask(true, true, false, true), 0x0b);
    }

    #[test]
    fn platform_windows_input_combines_utf16_surrogates_and_drops_invalid_halves() {
        let mut pending = None;
        assert_eq!(map_utf16_char(0xd83d, &mut pending), None);
        assert_eq!(
            map_utf16_char(0xde42, &mut pending),
            Some(Event::TextCommit(Arc::from("🙂")))
        );
        assert_eq!(pending, None);
        assert_eq!(map_utf16_char(0xde42, &mut pending), None);
        assert_eq!(map_utf16_char(0xd800, &mut pending), None);
        assert_eq!(
            map_utf16_char('A' as u16, &mut pending),
            Some(Event::TextCommit(Arc::from("A")))
        );
        assert_eq!(pending, None);
    }

    #[test]
    fn platform_windows_input_scales_signed_mouse_touch_and_pen_coordinates_once() {
        let scale = DeviceScale::new(2.0, 1.5).unwrap();
        let mouse = map_pointer_button(true, PointerDevice::Mouse, true, true, (-20, 30), scale);
        assert_eq!(
            mouse,
            Some(Event::PrimaryPointerDown {
                x_dip: -10.0,
                y_dip: 20.0,
            })
        );
        assert_eq!(
            signed_client_pixels((-12_i16 as u16 as u64 | (34_u16 as u64) << 16) as isize),
            (-12, 34)
        );
        assert_eq!(
            map_pointer_button(
                false,
                PointerDevice::Touch,
                true,
                true,
                (10, 12),
                DeviceScale::new(2.0, 3.0).unwrap(),
            ),
            Some(Event::PrimaryPointerUp {
                x_dip: 5.0,
                y_dip: 4.0,
            })
        );
        assert!(
            map_pointer_button(
                true,
                PointerDevice::Touch,
                false,
                true,
                (1, 2),
                DeviceScale::new(1.0, 1.0).unwrap(),
            )
            .is_none()
        );
        assert!(
            map_pointer_button(
                true,
                PointerDevice::Pen,
                true,
                false,
                (1, 2),
                DeviceScale::new(1.0, 1.0).unwrap(),
            )
            .is_none()
        );
        assert!(
            map_pointer_button(
                true,
                PointerDevice::Other,
                true,
                true,
                (1, 2),
                DeviceScale::new(1.0, 1.0).unwrap(),
            )
            .is_none()
        );
    }
}
