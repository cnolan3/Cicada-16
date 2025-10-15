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

#![allow(dead_code)]

#[derive(Debug, Copy, Clone)]
pub struct AddrCounter {
    pub physical_addr: u32,
    pub logical_addr: u32,
    pub num_bytes: u32,
    pub bank: u32,
}

impl AddrCounter {
    pub fn new() -> Self {
        Self {
            physical_addr: 0,
            logical_addr: 0,
            num_bytes: 0,
            bank: 0,
        }
    }

    pub fn increment_by(&mut self, inc: u32) {
        self.physical_addr += inc;
        self.logical_addr += inc;
        self.num_bytes += inc;
    }
}

#[derive(Debug)]
pub struct Context {
    pub name: Option<String>,
    pub size: Option<u32>,
    pub vaddr: Option<u32>,
    pub paddr: Option<u32>,
    pub align: Option<u32>,
    pub address: AddrCounter,
}

pub type ContextStack = Vec<Context>;
