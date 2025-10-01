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
}

// production file reader
pub struct AsmFileReader;

impl FileReader for AsmFileReader {
    fn read_to_string(&self, path: &Path) -> Result<String> {
        Ok(fs::read_to_string(path)?)
    }
}

// mock file reader for testing
#[derive(Default)]
pub struct MockFileReader {
    files: HashMap<PathBuf, String>,
}

impl MockFileReader {
    pub fn add_file(&mut self, path: &str, content: &str) {
        self.files.insert(PathBuf::from(path), content.to_string());
    }
}

impl FileReader for MockFileReader {
    fn read_to_string(&self, path: &Path) -> Result<String> {
        self.files
            .get(path)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Mock file not found: {}", path.display()))
    }
}
