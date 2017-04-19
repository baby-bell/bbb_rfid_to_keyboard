use std::process::{Command, ExitStatus};
use std::os::unix::process::ExitStatusExt;
use std::io::Error as IoError;
use std::error::Error;
use std::fmt;
use std::io::prelude::*;
use std::fs::OpenOptions;

#[derive(Debug)]
pub enum CommandError {
    NotExecuted(IoError),
    CommandFailed(String, ExitStatus),
}

impl CommandError {
    pub fn failure<S: Into<String>>(msg: S) -> CommandError {
        CommandError::CommandFailed(msg.into(), ExitStatus::from_raw(1))
    }
}

impl From<IoError> for CommandError {
    fn from(err: IoError) -> Self {
        CommandError::NotExecuted(err)
    }
}


impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::CommandError::*;
        match self {
            &NotExecuted(ref ioe) => write!(f, "could not exec command: {}", ioe)?,
            &CommandFailed(ref cmd, ref es) => write!(f, "{} -- command failed with exit status {}", cmd, es)?,
        }

        Ok(())
    }
}

impl Error for CommandError {
    fn description(&self) -> &str {
        match self {
            &CommandError::NotExecuted(_) => "Could not exec command",
            &CommandError::CommandFailed(..) => "Command returned failed status",
        }
    }
}

// Example at https://wiki.tizen.org/wiki/USB/Linux_USB_Layers/Configfs_Composite_Gadget/Usage_eq._to_g_hid.ko
pub static PRE_DESC_COMMANDS: &'static [(&'static str, &'static str)] = &[
    ("mkdir", "/config/usb_gadget/kb"),
    ("echo", "0x1234 > /config/usb_gadget/kb/idVendor"),
    ("echo", "0x5678 > /config/usb_gadget/kb/idProduct"),
    ("echo", "0x0100 > /config/usb_gadget/kb/bcdDevice"),
    ("echo", "0x0110 > /config/usb_gadget/kb/bcdUSB"),
    ("mkdir", "/config/usb_gadget/kb/configs/c.1"),
    ("mkdir", "/config/usb_gadget/kb/functions/hid.usb0"),
    ("echo", "1 > /config/usb_gadget/kb/functions/hid.usb0/subclass"),
    ("echo", "1 > /config/usb_gadget/kb/functions/hid.usb0/protocol"),
    ("echo", "8 > /config/usb_gadget/kb/functions/hid.usb0/report_length"),
];



pub static POST_DESC_COMMANDS: &'static [&'static str] = &[
    "ln",
    "echo",
];


pub static POST_DESC_ARGS: &'static [&'static [&'static str]] = &[
    &[ "-s", "/config/usb_gadget/kb/functions/hid.usb0",
    "/config/usb_gadget/kb/configs/c.1", ],
    &[ "musb-hdrc.0.auto > /config/usb_gadget/kb/UDC" ],
];

pub static DEINIT_COMMANDS: &'static [(&'static str, &'static str)] = &[
    ("rm", "/config/usb_gadget/kb/configs/c.1/hid.usb0/"),
    ("rmdir", "/config/usb_gadget/kb/functions/hid.usb0")
];

/// Handle echo commands that use `>`. 
fn handle_echo_redirects(args: &str) -> Result<(), CommandError> {
    use self::CommandError::CommandFailed;
    let mut split_iter = args.split(">");
    let (echopart, filepart) = (split_iter.next().unwrap(), split_iter.next().unwrap());
    match OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(filepart.trim()) {
        Ok(mut f) => {
            match f.write_all(echopart.trim().as_bytes()) {
                Ok(_) => Ok(()),
                Err(e) =>  {
                    Err(CommandError::failure(format!("echo could not write to file {} -- {}", filepart, e)))
                }
            }
        }
        Err(e) => {
            Err(CommandError::failure(format!("echo could not open file {} -- {}", filepart, e)))
        }
    }
}

/// Run `cmds`, feeding each a corresponding argument from `args`.
/// For code that uses redirects (`>`), the function assumes that
/// `echo` is being used, and will try to write the string on the left
/// side of the `>` to the file on the right.
pub fn run_commands(cmds: &[(&str, &str)]) -> Result<(), CommandError> {
    for &(cmd, arg) in cmds.into_iter() {
        let mut cmd = Command::new(cmd);
        if arg.contains(">") {
            handle_echo_redirects(arg)?;
            continue;
        }
        cmd.arg(arg);
        let cmd_string = format!("{:?}", cmd);
        let cmd_status = cmd.status()?;
        if !cmd_status.success() {
            return Err(CommandError::CommandFailed(cmd_string, cmd_status)); 
        }
    }
    Ok(())
}

pub fn run_post_desc_commands() -> Result<(), CommandError> {
    'top_loop:
    for (cmd, args) in POST_DESC_COMMANDS.into_iter().zip(POST_DESC_ARGS.into_iter()) {
        let mut cmd = Command::new(cmd);
        for arg in args.into_iter() {
            if arg.contains(">") {
                handle_echo_redirects(arg)?;
                continue 'top_loop;
            }
            cmd.arg(arg);
        }
        let cmd_string = format!("{:?}", cmd);
        let cmd_status = cmd.status()?;
        if !cmd_status.success() {
            return Err(CommandError::CommandFailed(cmd_string, cmd_status));
        }
    }
    Ok(())
}
