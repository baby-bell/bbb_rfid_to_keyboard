extern crate nix;
extern crate libc;

use libc::c_char;

use std::ffi::{CStr, CString};
use std::error::Error;
use std::io::prelude::*;
use std::fs::{File, OpenOptions};

mod commands;
use commands::{CommandError, PRE_DESC_COMMANDS, run_post_desc_commands, DEINIT_COMMANDS};

pub struct EmuKb {
	file_handle: File,
}

impl EmuKb {
    /// Initialize the emulated keyboard.
    pub fn init() -> Result<Self, KbError> {
        before_descriptor_commands()?;
        let descriptor_path = "/config/usb_gadget/kb/functions/hid.usb0/report_desc";
        let mut descriptor_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(descriptor_path)?;
        descriptor_file.write_all(GADGET_REPORT_DESCRIPTOR)?;
        println!("Wrote descriptor file.");
        run_post_desc_commands()?;
        println!("Ran post-descriptor commands.");
        let gadget_device = "/dev/hidg0";
        let gadget_file = match OpenOptions::new()
            .read(true)
            .write(true)
            .open(gadget_device) {
            Ok(f) => f,
            Err(e) => return Err(CommandError::failure(format!("could not open gadget device -- {}", e)).into())
        };
        Ok(EmuKb {
            file_handle: gadget_file
        })
    }
    /// Send `report` to the emulated keyboard.
    pub fn send_report(&mut self, report: &[u8]) -> Result<(), KbError> {
        self.file_handle.write_all(report)?;
        Ok(())
    }
}

#[repr(C)]
pub struct KbError {
	str_error: *const c_char,
}

impl<T> From<T> for KbError where T: Error {
	fn from(item: T) -> Self {
        let cstring = CString::new(format!("{}", item)).unwrap();
        KbError {
            str_error: cstring.into_raw() as *const _,
        }
	}
}

static GADGET_REPORT_DESCRIPTOR: &'static [u8] = &[
	0x05, 0x01,	/* USAGE_PAGE (Generic Desktop)	          */
	0x09, 0x06,	/* USAGE (Keyboard)                       */
	0xa1, 0x01,	/* COLLECTION (Application)               */
	0x05, 0x07,	/*   USAGE_PAGE (Keyboard)                */
	0x19, 0xe0,	/*   USAGE_MINIMUM (Keyboard LeftControl) */
	0x29, 0xe7,	/*   USAGE_MAXIMUM (Keyboard Right GUI)   */
	0x15, 0x00,	/*   LOGICAL_MINIMUM (0)                  */
	0x25, 0x01,	/*   LOGICAL_MAXIMUM (1)                  */
	0x75, 0x01,	/*   REPORT_SIZE (1)                      */
	0x95, 0x08,	/*   REPORT_COUNT (8)                     */
	0x81, 0x02,	/*   INPUT (Data,Var,Abs)                 */
	0x95, 0x01,	/*   REPORT_COUNT (1)                     */
	0x75, 0x08,	/*   REPORT_SIZE (8)                      */
	0x81, 0x03,	/*   INPUT (Cnst,Var,Abs)                 */
	0x95, 0x05,	/*   REPORT_COUNT (5)                     */
	0x75, 0x01,	/*   REPORT_SIZE (1)                      */
	0x05, 0x08,	/*   USAGE_PAGE (LEDs)                    */
	0x19, 0x01,	/*   USAGE_MINIMUM (Num Lock)             */
	0x29, 0x05,	/*   USAGE_MAXIMUM (Kana)                 */
	0x91, 0x02,	/*   OUTPUT (Data,Var,Abs)                */
	0x95, 0x01,	/*   REPORT_COUNT (1)                     */
	0x75, 0x03,	/*   REPORT_SIZE (3)                      */
	0x91, 0x03,	/*   OUTPUT (Cnst,Var,Abs)                */
	0x95, 0x06,	/*   REPORT_COUNT (6)                     */
	0x75, 0x08,	/*   REPORT_SIZE (8)                      */
	0x15, 0x00,	/*   LOGICAL_MINIMUM (0)                  */
	0x25, 0x65,	/*   LOGICAL_MAXIMUM (101)                */
	0x05, 0x07,	/*   USAGE_PAGE (Keyboard)                */
	0x19, 0x00,	/*   USAGE_MINIMUM (Reserved)             */
	0x29, 0x65,	/*   USAGE_MAXIMUM (Keyboard Application) */
	0x81, 0x00,	/*   INPUT (Data,Ary,Abs)                 */
	0xc0		/* END_COLLECTION                         */

];

fn before_descriptor_commands() -> Result<(), CommandError> {
    commands::run_commands(PRE_DESC_COMMANDS)
}

fn deinit_commands() {
    let _ = commands::run_commands(DEINIT_COMMANDS);
}

static USB2ASCII_ARRAY: &'static [u8] = &[
	  0,   0,   0,   0, b'a', b'b', b'c', b'd',
	b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l',
	b'm', b'n', b'o', b'p', b'q', b'r', b's', b't',
	b'u', b'v', b'w', b'x', b'y', b'z', b'1', b'2',
	b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'0',
	b'\n', 27, 7, b'\t', b' ', b'-', b'=', b'[',
	b']', b'\\', 0, b';', b'\'', b'`', b',', b'.',
	b'/',   0,  0,   0,   0,   0,   0,   0,
];

static USB2ASCII_ARRAY_SHIFTED: &'static [u8] = &[
	  0,   0,   0,   0, b'A', b'B', b'C', b'D',
	b'E', b'F', b'G', b'H', b'I', b'J', b'K', b'L',
	b'M', b'N', b'O', b'P', b'Q', b'R', b'S', b'T',
	b'U', b'V', b'W', b'X', b'Y', b'Z', b'1', b'2',
	b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'0',
	b'\n', 27, 7, b'\t', b' ', b'_', b'+', b'{',
	b'}', b'|', 0, b':', b'"', b'~', b'<', b'>',
	0,   0,  0,   0,   0,   0,   0,   0,
];

struct Keycode {
    pub modifiers: u8,
    pub code: u8
}

impl Keycode {
    /// Get a USB keycode from an ASCII character.
    pub fn from_ascii(character: u8) -> Self {
        let mut key = Keycode {
            modifiers: 0,
            code: 0
        };

        for (idx, ascii_char) in USB2ASCII_ARRAY.iter().enumerate() {
            if *ascii_char == character {
                key.code = idx as u8;
                return key;
            }
        }

        for (idx, ascii_char) in USB2ASCII_ARRAY_SHIFTED.iter().enumerate() {
            if *ascii_char == character {
                key.modifiers = 2;
                key.code = idx as u8;
                return key;
            }
        }

        key
    }
}


/// Reclain the contents of `err`. This function should
/// be called in lieu of `free`, since the Rust allocator
/// owns the data inside `err`.
#[no_mangle]
pub extern fn error_free(err: KbError) {
	if err.str_error.is_null() {
		return;
	}

	unsafe {
		let string = CString::from_raw(err.str_error as *mut _);
		drop(string);
	}
}

/// Intialize `emukb_out`, which should be a pointer to `NULL`. If
/// an error comes up and `error` is not `NULL`, then the function
/// returns false and sets `error` to convey information.
#[no_mangle]
pub extern fn emukb_init(emukb_out: *mut *mut EmuKb, error: *mut KbError) -> bool {
	if emukb_out.is_null() || error.is_null() {
		return false;
	}
    match EmuKb::init() {
        Ok(kb) => {
            unsafe { *emukb_out = Box::into_raw(Box::new(kb)); }
            return true;
        }
        Err(e) => {
            unsafe { *error = e; }
            //deinit_commands();
            return false;
        }
    }
}

/// Deinitialize `keyboard`. This function ignores
/// errors that might arise.
#[no_mangle]
pub extern fn emukb_deinit(keyboard: *mut EmuKb) {
    if keyboard.is_null() {
        return;
    }

    unsafe {
        let boxed = Box::from_raw(keyboard);
        drop(boxed);
    }
    deinit_commands();
}

/// Send `string` using `keyboard`.
#[no_mangle]
pub extern fn emukb_send_string(keyboard: *mut EmuKb, string: *const c_char,
    error: *mut KbError) -> bool {
    if keyboard.is_null() || error.is_null() || string.is_null() {
        return false;
    }

    let (kb, string) = unsafe {
        (&mut *keyboard, CStr::from_ptr(string))
    };


    let mut report: [u8; 8];
    for byte in string.to_bytes() {
        report = [0u8; 8];
        let keycode = Keycode::from_ascii(*byte);
        report[2] = keycode.code;
        report[0] = keycode.modifiers;

        if let Err(e) = kb.send_report(&report) {
            unsafe { *error = e.into(); }
            return false;
        }

        report = [0u8; 8];
        if let Err(e) = kb.send_report(&report) {
            unsafe { *error = e.into(); }
            return false;
        }
    }

    return true;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
