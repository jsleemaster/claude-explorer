use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use portable_pty::{native_pty_system, CommandBuilder, PtySize, PtyPair};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct TerminalPane {
    pty_pair: Option<PtyPair>,
    output_buffer: Arc<Mutex<Vec<u8>>>,
    input_buffer: String,
    scroll_offset: usize,
    cwd: std::path::PathBuf,
}

impl TerminalPane {
    pub fn new(cwd: &Path) -> anyhow::Result<Self> {
        let output_buffer = Arc::new(Mutex::new(Vec::new()));

        // Create PTY
        let pty_system = native_pty_system();
        let pty_pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        // Spawn claude process
        let mut cmd = CommandBuilder::new("claude");
        cmd.cwd(cwd);

        let mut child = pty_pair.slave.spawn_command(cmd)?;

        // Read output in background thread
        let mut reader = pty_pair.master.try_clone_reader()?;
        let buffer_clone = Arc::clone(&output_buffer);

        thread::spawn(move || {
            let mut buf = [0u8; 1024];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let mut buffer = buffer_clone.lock().unwrap();
                        buffer.extend_from_slice(&buf[..n]);
                        // Keep buffer size reasonable
                        if buffer.len() > 1_000_000 {
                            let drain_to = buffer.len() - 500_000;
                            buffer.drain(..drain_to);
                        }
                    }
                    Err(_) => break,
                }
            }
            let _ = child.wait();
        });

        Ok(Self {
            pty_pair: Some(pty_pair),
            output_buffer,
            input_buffer: String::new(),
            scroll_offset: 0,
            cwd: cwd.to_path_buf(),
        })
    }

    pub fn tick(&mut self) {
        // Called on each tick - can be used for animations or updates
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if let Some(ref pty_pair) = self.pty_pair {
            let bytes = match (key.code, key.modifiers) {
                (KeyCode::Char(c), KeyModifiers::NONE) => vec![c as u8],
                (KeyCode::Char(c), KeyModifiers::SHIFT) => vec![c.to_ascii_uppercase() as u8],
                (KeyCode::Char(c), KeyModifiers::CONTROL) => {
                    // Ctrl+A = 1, Ctrl+B = 2, etc.
                    let ctrl_char = (c.to_ascii_lowercase() as u8).wrapping_sub(b'a' - 1);
                    vec![ctrl_char]
                }
                (KeyCode::Enter, _) => vec![b'\r'],
                (KeyCode::Backspace, _) => vec![127],
                (KeyCode::Delete, _) => vec![27, b'[', b'3', b'~'],
                (KeyCode::Tab, _) => vec![b'\t'],
                (KeyCode::Up, _) => vec![27, b'[', b'A'],
                (KeyCode::Down, _) => vec![27, b'[', b'B'],
                (KeyCode::Right, _) => vec![27, b'[', b'C'],
                (KeyCode::Left, _) => vec![27, b'[', b'D'],
                (KeyCode::Home, _) => vec![27, b'[', b'H'],
                (KeyCode::End, _) => vec![27, b'[', b'F'],
                (KeyCode::PageUp, _) => vec![27, b'[', b'5', b'~'],
                (KeyCode::PageDown, _) => vec![27, b'[', b'6', b'~'],
                (KeyCode::Esc, _) => vec![27],
                _ => return,
            };

            if let Ok(mut writer) = pty_pair.master.try_clone_writer() {
                let _ = writer.write_all(&bytes);
                let _ = writer.flush();
            }
        }
    }

    pub fn send_interrupt(&mut self) {
        if let Some(ref pty_pair) = self.pty_pair {
            if let Ok(mut writer) = pty_pair.master.try_clone_writer() {
                let _ = writer.write_all(&[3]); // Ctrl+C
                let _ = writer.flush();
            }
        }
    }

    pub fn insert_text(&mut self, text: &str) {
        if let Some(ref pty_pair) = self.pty_pair {
            if let Ok(mut writer) = pty_pair.master.try_clone_writer() {
                let _ = writer.write_all(text.as_bytes());
                let _ = writer.flush();
            }
        }
    }

    pub fn change_directory(&mut self, path: &Path) {
        // Send cd command to the terminal
        let cmd = format!("cd {}\r", path.display());
        self.insert_text(&cmd);
        self.cwd = path.to_path_buf();
    }

    pub fn get_output(&self) -> String {
        let buffer = self.output_buffer.lock().unwrap();
        String::from_utf8_lossy(&buffer).to_string()
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(3);
    }

    pub fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(3);
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn resize(&mut self, cols: u16, rows: u16) {
        if let Some(ref pty_pair) = self.pty_pair {
            let _ = pty_pair.master.resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            });
        }
    }
}

impl Drop for TerminalPane {
    fn drop(&mut self) {
        // PTY will be cleaned up automatically
        self.pty_pair.take();
    }
}
