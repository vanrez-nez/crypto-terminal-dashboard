use std::ffi::CString;

/// Keyboard events for navigation
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyEvent {
    Up,
    Down,
    Left,
    Right,
    Enter,
    Escape,
    Tab,
    ShiftTab,
    PageUp,
    PageDown,
    Home,
    End,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Space,
    Char(char),
}

/// Linux input event key codes
mod keycodes {
    pub const KEY_ESC: u16 = 1;
    pub const KEY_1: u16 = 2;
    pub const KEY_2: u16 = 3;
    pub const KEY_3: u16 = 4;
    pub const KEY_4: u16 = 5;
    pub const KEY_5: u16 = 6;
    pub const KEY_Q: u16 = 16;
    pub const KEY_W: u16 = 17;
    pub const KEY_R: u16 = 19;
    pub const KEY_H: u16 = 35;
    pub const KEY_J: u16 = 36;
    pub const KEY_K: u16 = 37;
    pub const KEY_L: u16 = 38;
    pub const KEY_C: u16 = 46;
    pub const KEY_M: u16 = 50;
    pub const KEY_SPACE: u16 = 57;
    pub const KEY_TAB: u16 = 15;
    pub const KEY_ENTER: u16 = 28;
    pub const KEY_LEFTSHIFT: u16 = 42;
    pub const KEY_RIGHTSHIFT: u16 = 54;
    pub const KEY_HOME: u16 = 102;
    pub const KEY_UP: u16 = 103;
    pub const KEY_PAGEUP: u16 = 104;
    pub const KEY_LEFT: u16 = 105;
    pub const KEY_RIGHT: u16 = 106;
    pub const KEY_END: u16 = 107;
    pub const KEY_DOWN: u16 = 108;
    pub const KEY_PAGEDOWN: u16 = 109;
}

/// Input event structure from Linux evdev
#[repr(C)]
struct InputEvent {
    tv_sec: libc::time_t,
    tv_usec: libc::suseconds_t,
    type_: u16,
    code: u16,
    value: i32,
}

const EV_KEY: u16 = 1;
const KEY_PRESS: i32 = 1;
// const KEY_RELEASE: i32 = 0;
// const KEY_REPEAT: i32 = 2;

/// Keyboard input handler using Linux evdev
pub struct KeyboardInput {
    device_fd: Option<i32>,
    shift_held: bool,
}

// EVIOCGBIT ioctl to get event bits
// On Linux: _IOC(_IOC_READ, 'E', 0x20 + ev_type, len)
// For EV_KEY (type 1): _IOC(2, ord('E'), 0x21, len)
// = (2 << 30) | (len << 16) | (ord('E') << 8) | 0x21
// For 96 bytes: (2 << 30) | (96 << 16) | (0x45 << 8) | 0x21 = 0x80604521

// EVIOCGBIT for EV (event types)
// = (2 << 30) | (32 << 16) | (0x45 << 8) | 0x20 = 0x80204520
const EVIOCGBIT_EV: libc::c_ulong = 0x80204520;

// EVIOCGBIT for EV_KEY (key capabilities) - 96 bytes for key bitmap
// = (2 << 30) | (96 << 16) | (0x45 << 8) | 0x21 = 0x80604521
const EVIOCGBIT_KEY: libc::c_ulong = 0x80604521;

// Key codes for detecting real keyboards (vs HDMI CEC, etc)
const KEY_A: u16 = 30;
const KEY_Z: u16 = 44;
const KEY_SPACE: u16 = 57;

/// Check if a bit is set in a byte array
fn has_key(bits: &[u8], key: u16) -> bool {
    let byte_idx = (key / 8) as usize;
    let bit_idx = key % 8;
    if byte_idx < bits.len() {
        (bits[byte_idx] & (1 << bit_idx)) != 0
    } else {
        false
    }
}

impl KeyboardInput {
    pub fn new() -> Self {
        // Try to find a real keyboard device (not HDMI CEC, etc)
        for i in 0..10 {
            let path = format!("/dev/input/event{}", i);
            let c_path = CString::new(path.clone()).unwrap();
            let fd = unsafe { libc::open(c_path.as_ptr(), libc::O_RDONLY | libc::O_NONBLOCK) };

            if fd >= 0 {
                // Check if this device supports EV_KEY events
                let mut ev_bits = [0u8; 32];
                let ret = unsafe { libc::ioctl(fd, EVIOCGBIT_EV, ev_bits.as_mut_ptr()) };

                // Check if EV_KEY (bit 1) is set in event types
                let has_key_events = ret >= 0 && (ev_bits[0] & 0x02) != 0;

                if has_key_events {
                    // Now check if this is a REAL keyboard by looking for letter keys
                    // HDMI CEC and other devices have EV_KEY but not letter keys
                    let mut key_bits = [0u8; 96];
                    let ret = unsafe { libc::ioctl(fd, EVIOCGBIT_KEY, key_bits.as_mut_ptr()) };

                    if ret >= 0 {
                        // A real keyboard will have A, Z, and Space keys
                        let is_keyboard = has_key(&key_bits, KEY_A)
                            && has_key(&key_bits, KEY_Z)
                            && has_key(&key_bits, KEY_SPACE);

                        if is_keyboard {
                            println!("Using keyboard: {} (has letter keys)", path);
                            return KeyboardInput {
                                device_fd: Some(fd),
                                shift_held: false,
                            };
                        } else {
                            println!("Skipping {}: has EV_KEY but not a keyboard", path);
                        }
                    }
                }
                unsafe {
                    libc::close(fd);
                }
            }
        }

        println!("No keyboard found, input disabled");
        KeyboardInput {
            device_fd: None,
            shift_held: false,
        }
    }

    /// Poll for keyboard events (non-blocking)
    pub fn poll_events(&mut self) -> Vec<KeyEvent> {
        let mut events = Vec::new();

        let Some(fd) = self.device_fd else {
            return events;
        };

        let event_size = std::mem::size_of::<InputEvent>();
        let mut event: InputEvent = unsafe { std::mem::zeroed() };

        loop {
            let ret =
                unsafe { libc::read(fd, &mut event as *mut _ as *mut libc::c_void, event_size) };

            if ret != event_size as isize {
                break;
            }

            // Only handle key events
            if event.type_ != EV_KEY {
                continue;
            }

            // Track shift state
            if event.code == keycodes::KEY_LEFTSHIFT || event.code == keycodes::KEY_RIGHTSHIFT {
                self.shift_held = event.value != 0; // 1 = pressed, 0 = released, 2 = repeat
                continue;
            }

            // Only handle key press events (not release or repeat)
            if event.value != KEY_PRESS {
                continue;
            }

            // Debug: log all key press events
            println!("[DEBUG] Key pressed: code={}, value={}", event.code, event.value);

            // Map key codes to events
            let key_event = match event.code {
                keycodes::KEY_ESC => Some(KeyEvent::Escape),
                keycodes::KEY_ENTER => Some(KeyEvent::Enter),
                keycodes::KEY_TAB => {
                    if self.shift_held {
                        Some(KeyEvent::ShiftTab)
                    } else {
                        Some(KeyEvent::Tab)
                    }
                }
                keycodes::KEY_UP => Some(KeyEvent::Up),
                keycodes::KEY_DOWN => Some(KeyEvent::Down),
                keycodes::KEY_LEFT => Some(KeyEvent::Left),
                keycodes::KEY_RIGHT => Some(KeyEvent::Right),
                keycodes::KEY_PAGEUP => Some(KeyEvent::PageUp),
                keycodes::KEY_PAGEDOWN => Some(KeyEvent::PageDown),
                keycodes::KEY_HOME => Some(KeyEvent::Home),
                keycodes::KEY_END => Some(KeyEvent::End),
                keycodes::KEY_1 => Some(KeyEvent::Num1),
                keycodes::KEY_2 => Some(KeyEvent::Num2),
                keycodes::KEY_3 => Some(KeyEvent::Num3),
                keycodes::KEY_4 => Some(KeyEvent::Num4),
                keycodes::KEY_5 => Some(KeyEvent::Num5),
                keycodes::KEY_SPACE => Some(KeyEvent::Space),
                // Character keys
                keycodes::KEY_Q => Some(KeyEvent::Char('q')),
                keycodes::KEY_W => Some(KeyEvent::Char('w')),
                keycodes::KEY_R => Some(KeyEvent::Char('r')),
                keycodes::KEY_H => Some(KeyEvent::Char('h')),
                keycodes::KEY_J => Some(KeyEvent::Char('j')),
                keycodes::KEY_K => Some(KeyEvent::Char('k')),
                keycodes::KEY_L => Some(KeyEvent::Char('l')),
                keycodes::KEY_C => Some(KeyEvent::Char('c')),
                keycodes::KEY_M => Some(KeyEvent::Char('m')),
                _ => None,
            };

            if let Some(e) = key_event {
                events.push(e);
            }
        }

        events
    }

    /// Check if escape was pressed (for quick exit check)
    pub fn is_escape_pressed(&mut self) -> bool {
        self.poll_events().contains(&KeyEvent::Escape)
    }
}

impl Drop for KeyboardInput {
    fn drop(&mut self) {
        if let Some(fd) = self.device_fd {
            unsafe {
                libc::close(fd);
            }
        }
    }
}

impl Default for KeyboardInput {
    fn default() -> Self {
        Self::new()
    }
}
