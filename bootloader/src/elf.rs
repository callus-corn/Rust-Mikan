extern crate alloc;

use alloc::vec::Vec;
use core::iter::Iterator;

pub struct Elf {
    elf_header: ElfHeader,
    program_headers: Vec<ProgramHeader>,
}

impl Elf {
    pub fn new(buffer: &mut [u8]) -> Elf {
        let elf_header = ElfHeader::new(buffer);
        
        let mut program_headers = Vec::new();
        let program_header_addr = buffer.as_ptr() as u64 + elf_header.e_phoff;
        for i in 0..elf_header.e_phnum {
            let program_header_ptr = (program_header_addr + i as u64 * elf_header.e_phentsize as u64) as *const ProgramHeader;
            let program_header = unsafe { *program_header_ptr };
            program_headers.push(program_header);
        }
        Elf {
            elf_header,
            program_headers,
        }
    }

    pub fn calculate_base_addr(&self) -> u64 {
        let mut base_addr = u64::MAX;
        for program_header in self.program_headers.iter() {
            if !program_header.type_is_load() {
                continue;
            }
            if base_addr > program_header.p_vaddr {
                base_addr = program_header.p_vaddr;
            }
        }
        base_addr
    }

    pub fn calculate_page_count(&self) -> usize {
        let mut base_addr = u64::MAX;
        let mut last_addr = 0;
        for program_header in self.program_headers.iter() {
            if !program_header.type_is_load() {
                continue;
            }
            if base_addr > program_header.p_vaddr {
                base_addr = program_header.p_vaddr;
            }
            if last_addr < program_header.p_vaddr + program_header.p_memsz {
                last_addr = program_header.p_vaddr + program_header.p_memsz;
            }
        }
        ((last_addr - base_addr + 0xfff) / 0x1000) as usize
    }

    pub fn program_header_iter(&self) -> impl Iterator<Item = &ProgramHeader> {
        self.program_headers.iter()
    }

    pub fn entry(&self) -> u64 {
        self.elf_header.e_entry
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
struct ElfHeader {
    e_ident: [u8; 16],
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: u64,
    e_phoff: u64,
    e_shoff: u64,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

impl ElfHeader {
    fn new(buffer: &mut [u8]) -> ElfHeader {
        let elf_header_ptr = buffer.as_ptr() as *const ElfHeader;
        unsafe { *elf_header_ptr }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ProgramHeader {
    p_type: u32,
    p_flags: u32,
    p_offset: u64,
    p_vaddr: u64,
    p_paddr: u64,
    p_filesz: u64,
    p_memsz: u64,
    p_align: u64,
}

impl ProgramHeader {
    pub fn type_is_load(&self) -> bool {
        self.p_type == 1
    }

    pub fn p_vaddr(&self) -> u64 {
        self.p_vaddr
    }

    pub fn p_offset(&self) -> u64 {
        self.p_offset
    }

    pub fn p_memsz(&self) -> u64 {
        self.p_memsz
    }
}
