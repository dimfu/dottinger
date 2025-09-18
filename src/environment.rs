use std::fs::File;
use std::io::{BufRead, Cursor, Read};
use std::{collections::HashMap, fs::OpenOptions, io};

pub struct Environment {
    pub map: HashMap<String, Vec<u8>>,
    buf: Vec<u8>,
    file: Option<File>,
    path: String,
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

        let env_content = String::from_utf8_lossy(&self.buf);
        for line in env_content.lines() {
            if let Some((key, value)) = line.split_once("=") {
                self.map
                    .insert(String::from(key), value.as_bytes().to_vec());
            }
        }

        Ok(())
    }

    fn key_exists(&self, key: &String) -> bool {
        self.map.contains_key(key)
    }

    pub fn get_with_key(&self, key: &String) -> io::Result<String> {
        match self.map.get(key) {
            Some(val) => {
                let s = String::from_utf8(val.clone())
                    .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid UTF-8"))?;
                Ok(s)
            }
            None => Err(io::Error::new(io::ErrorKind::NotFound, "key doesn't exist")),
        }
    }

    // update an existing environment key with new value
    pub fn set(&mut self, target_key: &String, new_value: Vec<u8>) -> io::Result<()> {
        if !self.key_exists(target_key) {
            return Err(io::Error::new(io::ErrorKind::NotFound, "key doesn't exist"));
        }

        let mut cursor = Cursor::new(&self.buf);
        let mut offset = 0;
        let mut line = String::new();

        // find the exact key buf position
        loop {
            line.clear();
            let n = match cursor.read_line(&mut line) {
                Ok(0) | Err(_) => break,
                Ok(n) => n,
            };

            let line_bytes = &self.buf[offset..offset + n];
            let line_str = String::from_utf8_lossy(line_bytes);

            if let Some((key, _)) = line_str.trim_end().split_once('=') {
                if key == target_key {
                    let value_start = offset + key.len() + 1;
                    let value_end = offset + line_str.trim_end().len();

                    self.buf
                        .splice(value_start..value_end, new_value.iter().cloned());
                    break;
                }
            }

            offset += n;
        }

        self.write_buf()?;
        return Ok(());
    }
}
