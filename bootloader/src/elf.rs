extern crate alloc;

use core::iter::Iterator;
use alloc::vec::Vec;

pub struct ElfHeader {
    //面倒なのでウソ実装
    pub entry: u64,
}

impl ElfHeader {
    pub fn new(buffer: &mut [u8]) -> ElfHeader {
        let entry_ptr = (buffer.as_ptr() as u64 + 24) as *const u64;
        //elfの仕様どおりに作っているつもりだが安全性は不明
        let entry = unsafe { *entry_ptr };
        ElfHeader {
            entry: entry,
        }
    }
}

#[derive(Copy, Clone)]
pub struct ProgramHeader {
    //嘘実装
    pub paddr: u64,
    pub offset: u64,
    pub memsz: u64,
}

pub struct ProgramHeaderIter {
    program_headers: Vec<ProgramHeader>,
    index: u16,
    len: u16,
}

impl ProgramHeaderIter {
    pub fn new(buffer: &mut [u8]) -> ProgramHeaderIter {
        let mut program_headers = Vec::new();
        let ph_offset_ptr = (buffer.as_ptr() as u64 + 32) as *const u64;
        //elfの仕様どおりに作っているつもりだが安全性は不明
        let ph_offset = unsafe { *ph_offset_ptr };
        let program_header_size_ptr = (buffer.as_ptr() as u64 + 54) as *const u16;
        //elfの仕様どおりに作っているつもりだが安全性は不明
        let program_header_size = unsafe { *program_header_size_ptr };
        let len_ptr = (buffer.as_ptr() as u64 + 56) as *const u16;
        //elfの仕様どおりに作っているつもりだが安全性は不明
        let len = unsafe { *len_ptr };

        for i in 0..len {
            let file_offset = ph_offset + (i*program_header_size) as u64;
            let offset_ptr = (buffer.as_ptr() as u64 + file_offset + 8 ) as *const u64;
            //elfの仕様どおりに作っているつもりだが安全性は不明
            let offset = unsafe { *offset_ptr };
            let phys_addr_ptr = (buffer.as_ptr() as  u64 + file_offset + 24 ) as *const u64;
            //elfの仕様どおりに作っているつもりだが安全性は不明
            let phys_addr = unsafe { *phys_addr_ptr };
            let memory_size_ptr = (buffer.as_ptr() as  u64 + file_offset + 40 ) as *const u64;
            //elfの仕様どおりに作っているつもりだが安全性は不明
            let memory_size = unsafe { *memory_size_ptr };
            program_headers.push(ProgramHeader{
                paddr: phys_addr,
                offset: offset,
                memsz: memory_size,
            });
        }

        ProgramHeaderIter{
            program_headers: program_headers,
            index: 0,
            len: len,
        }
    }
}

impl Iterator for ProgramHeaderIter {
    type Item = ProgramHeader;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            let program_header = self.program_headers[self.index as usize];
            self.index += 1;
            Some(program_header)
        } else {
            None
        }
    }
}
