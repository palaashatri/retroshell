#![allow(dead_code)]

use nix::pty::{openpty, Winsize};
use nix::unistd::{dup2, execve, fork, setsid, ForkResult, Pid};
use std::ffi::CString;
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd};

pub struct Pty {
    pub master_file: File,
}

impl Pty {
    pub fn new(cols: u16, rows: u16) -> Result<(Self, Pid), String> {
        let ws = Winsize {
            ws_row: rows,
            ws_col: cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        let openpty_res =
            openpty(Some(&ws), None).map_err(|e| format!("Failed to openpty: {}", e))?;

        let master_fd = openpty_res.master;
        let slave_fd = openpty_res.slave;

        match unsafe { fork() } {
            Ok(ForkResult::Parent { child }) => {
                let master_file = unsafe { File::from_raw_fd(master_fd.into_raw_fd()) };
                // Close slave fd by dropping it
                drop(slave_fd);
                Ok((Pty { master_file }, child))
            }
            Ok(ForkResult::Child) => {
                setsid().map_err(|e| format!("setsid failed: {}", e))?;

                let slave_raw_fd = slave_fd.into_raw_fd();
                dup2(slave_raw_fd, 0).map_err(|e| format!("dup2 stdin failed: {}", e))?;
                dup2(slave_raw_fd, 1).map_err(|e| format!("dup2 stdout failed: {}", e))?;
                dup2(slave_raw_fd, 2).map_err(|e| format!("dup2 stderr failed: {}", e))?;

                let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
                let shell_c = CString::new(shell.clone()).unwrap();
                let args = [shell_c.clone()];
                let env: [CString; 0] = [];

                let _ = execve(&shell_c, &args, &env);
                std::process::exit(1);
            }
            Err(e) => Err(format!("Fork failed: {}", e)),
        }
    }

    pub fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.master_file.read(buf)
    }

    pub fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.master_file.write(buf)
    }

    pub fn resize(&self, cols: u16, rows: u16) -> std::io::Result<()> {
        let ws = Winsize {
            ws_row: rows,
            ws_col: cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        unsafe {
            let fd = self.master_file.as_raw_fd();
            if libc::ioctl(fd, libc::TIOCSWINSZ, &ws) < 0 {
                return Err(std::io::Error::last_os_error());
            }
        }
        Ok(())
    }
}
