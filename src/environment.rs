use std::fs::File;
use std::io::{BufRead, Cursor, Read};
use std::{collections::HashMap, fmt, fs::OpenOptions, io};

pub enum KeyStatus {
    Enable,
    Disable,
}

pub struct Environment {
    pub map: HashMap<String, (usize, usize, usize)>,
    buf: Vec<u8>,
    file: Option<File>,
    path: String,
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (key, (_, start, end)) in &self.map {
            if let Ok(s) = std::str::from_utf8(&self.buf[*start..*end]) {
                writeln!(f, "{} = {}", key, s)?;
            } else {
                writeln!(f, "{} = <invalid utf-8>", key)?;
            }
        }
        Ok(())
    }
}

impl Environment {
    pub fn new(path: &String) -> Self {
        Environment {
            map: HashMap::new(),
            buf: Vec::new(),
            file: None,
            path: path.clone(),
        }
    }

    fn write_buf(&mut self) -> io::Result<()> {
        if let Some(file) = &mut self.file {
            use std::io::{Seek, SeekFrom, Write};
            file.seek(SeekFrom::Start(0))?;
            file.write_all(&self.buf)?;
            file.set_len(self.buf.len() as u64)?;
        }
        Ok(())
    }

    pub fn read_buf(&mut self) -> io::Result<()> {
        let mut file = match OpenOptions::new().read(true).write(true).open(&self.path) {
            Err(e) => return Err(e),
            Ok(f) => f,
        };

        file.read_to_end(&mut self.buf)?;
        self.file = Some(file);

        let mut cursor = Cursor::new(&self.buf);
        let mut line_offset = 0;
        let mut line = String::new();

        loop {
            line.clear();
            let n = match cursor.read_line(&mut line) {
                Ok(0) | Err(_) => break,
                Ok(n) => n,
            };

            let line_bytes = &self.buf[line_offset..line_offset + n];
            let line_str = String::from_utf8_lossy(line_bytes);

            if let Some((key, _)) = line_str.trim_end().split_once('=') {
                let cleaned_key = key.trim_start_matches(|c: char| c == '#' || c.is_whitespace());
                let value_start = line_offset + key.len() + 1;
                let value_end = line_offset + line_str.trim_end().len();

                self.map.insert(
                    String::from(cleaned_key),
                    (line_offset, value_start, value_end),
                );
            }

            line_offset += n;
        }

        Ok(())
    }

    pub fn get_with_key(&self, key: &String) -> io::Result<String> {
        match self.map.get(key) {
            Some((_, start, end)) => match String::from_utf8(self.buf[*start..*end].to_vec()) {
                Ok(s) => Ok(s),
                Err(_) => Err(io::Error::new(io::ErrorKind::InvalidData, "invalid utf-8")),
            },
            None => Err(io::Error::new(io::ErrorKind::NotFound, "doesn't exist")),
        }
    }

    pub fn set(&mut self, key: &String, new_value: Vec<u8>) -> io::Result<()> {
        let (start, end) = match self.map.get(key) {
            Some((_, start, end)) => (*start, *end),
            None => {
                // add new line to last buf since we're adding new variable
                self.buf.push(b'\n');
                (self.buf.len(), self.buf.len())
            }
        };

        // handle new variable creation, otherwise update the existing variable's value
        if start == end {
            let mut line = format!("{}=", key).into_bytes();
            line.extend_from_slice(&new_value);
            self.buf.extend(line);
        } else {
            self.buf.splice(start..end, new_value.iter().cloned());
        }

        self.write_buf()?;

        Ok(())
    }

    pub fn toggle(&mut self, key: &String, status: KeyStatus) -> io::Result<()> {
        let line_offset = match self.map.get(key) {
            Some((line_offset, _, _)) => *line_offset,
            None => {
                return Err(io::Error::new(io::ErrorKind::NotFound, "doesn't exist"));
            }
        };

        match status {
            KeyStatus::Disable => {
                self.buf
                    .splice(line_offset..line_offset, b"#".iter().cloned());
            }
            KeyStatus::Enable => {
                // check if the current line offset is a comment
                if self.buf.get(line_offset) == Some(&b'#') {
                    self.buf
                        .splice(line_offset..line_offset + 1, std::iter::empty());
                } else {
                    return Err(io::Error::new(io::ErrorKind::NotFound, "already enabled"));
                }
            }
        }

        self.write_buf()?;

        Ok(())
    }
}
