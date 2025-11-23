/*
Copyright 2025 Connor Nolan

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub trait FileReader {
    fn read_to_string(&self, path: &Path) -> Result<String>;
    fn read_binary(&self, path: &Path) -> Result<Vec<u8>>;
}

// production file reader
pub struct AsmFileReader;

impl FileReader for AsmFileReader {
    fn read_to_string(&self, path: &Path) -> Result<String> {
        Ok(fs::read_to_string(path)?)
    }

    fn read_binary(&self, path: &Path) -> Result<Vec<u8>> {
        Ok(fs::read(path)?)
    }
}

// Enum to hold either text or binary data
#[derive(Clone)]
enum FileData {
    Text(String),
    Binary(Vec<u8>),
}

// mock file reader for testing
#[derive(Default)]
pub struct MockFileReader {
    files: HashMap<PathBuf, FileData>,
}

impl MockFileReader {
    pub fn add_file(&mut self, path: &str, content: &str) {
        self.files
            .insert(PathBuf::from(path), FileData::Text(content.to_string()));
    }

    pub fn add_binary_file(&mut self, path: &str, content: &[u8]) {
        self.files
            .insert(PathBuf::from(path), FileData::Binary(content.to_vec()));
    }
}

impl FileReader for MockFileReader {
    fn read_to_string(&self, path: &Path) -> Result<String> {
        match self.files.get(path) {
            Some(FileData::Text(content)) => Ok(content.clone()),
            Some(FileData::Binary(_)) => {
                Err(anyhow::anyhow!("Cannot read binary file as string: {}", path.display()))
            }
            None => Err(anyhow::anyhow!("Mock file not found: {}", path.display())),
        }
    }

    fn read_binary(&self, path: &Path) -> Result<Vec<u8>> {
        match self.files.get(path) {
            Some(FileData::Binary(content)) => Ok(content.clone()),
            Some(FileData::Text(_)) => {
                Err(anyhow::anyhow!("Cannot read text file as binary: {}", path.display()))
            }
            None => Err(anyhow::anyhow!("Mock file not found: {}", path.display())),
        }
    }
}
