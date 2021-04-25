#![feature(abi_efiapi)]
#![feature(alloc_error_handler)]
#![no_std]
#![no_main]

extern crate alloc;

use bootloader::elf::{ElfHeader, ProgramHeaderIter};
use bootloader::vga::Writer;
use core::alloc::Layout;
use core::fmt::Write;
use core::mem;
use core::panic::PanicInfo;
use core::slice;
use uefi::prelude::*;
use uefi::proto::console::gop::{FrameBuffer, GraphicsOutput};
use uefi::proto::media::file;
use uefi::proto::media::file::{File, FileAttribute, FileMode, RegularFile};
use uefi::table::boot::{AllocateType, MemoryAttribute, MemoryType};

#[alloc_error_handler]
fn on_oom(_layout: Layout) -> ! {
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    //vga(https://github.com/phil-opp/blog_os/blob/post-03/src/vga_buffer.rs)
    let mut writer = Writer::new();
    writeln!(writer, "{}", info).unwrap();
    loop {}
}

#[entry]
fn efi_main(handle: Handle, system_table: SystemTable<Boot>) -> Status {
    let boot_services = system_table.boot_services();
    let stdout = system_table.stdout();

    //uefi-rsのextsフィーチャー(feature = exts)を使うために初期化が必要
    //extsフィーチャーを使わないならいらない
    unsafe {
        uefi::alloc::init(boot_services);
    }

    writeln!(stdout, "Hello, world!").unwrap();

    //バッファサイズは適当
    let memory_map_buffer = &mut [0; 4096 * 4];
    //このResultはuefi-rs独自実装のためunwrap_successを使う。
    let (_memory_map_key, descriptor_iter) =
        boot_services.memory_map(memory_map_buffer).unwrap_success();

    //uefi-rsの拡張機能(feature = exts)
    let file_system = boot_services
        .get_image_file_system(handle)
        .unwrap_success()
        .get();
    //生ポインタ解決
    //uefiから返ってくるstatusがsuccess以外だとpanicが呼ばれるため安全なポインタのはず
    let mut root_dir = unsafe { (*file_system).open_volume().unwrap_success() };

    let memory_map_file_handle = root_dir
        .open(
            "\\memmap",
            FileMode::CreateReadWrite,
            FileAttribute::empty(),
        )
        .unwrap_success();
    //RegularFileに変換する必要あり(unsafe)
    //理由はよくわかっていない
    //安全性もよくわからない
    let mut memory_map_file = unsafe { RegularFile::new(memory_map_file_handle) };

    memory_map_file
        .write("Type, PhysicalStart, NumberOfPages, Attribute\n".as_bytes())
        .unwrap_success();
    for descriptor in descriptor_iter {
        let memory_type: u32 = match descriptor.ty {
            MemoryType::RESERVED => 0,
            MemoryType::LOADER_CODE => 1,
            MemoryType::LOADER_DATA => 2,
            MemoryType::BOOT_SERVICES_CODE => 3,
            MemoryType::BOOT_SERVICES_DATA => 4,
            MemoryType::RUNTIME_SERVICES_CODE => 5,
            MemoryType::RUNTIME_SERVICES_DATA => 6,
            MemoryType::CONVENTIONAL => 7,
            MemoryType::UNUSABLE => 8,
            MemoryType::ACPI_RECLAIM => 9,
            MemoryType::ACPI_NON_VOLATILE => 10,
            MemoryType::MMIO => 11,
            MemoryType::MMIO_PORT_SPACE => 12,
            MemoryType::PAL_CODE => 13,
            MemoryType::PERSISTENT_MEMORY => 14,
            _ => 0xffff_ffff,
        };
        let physical_start = descriptor.phys_start;
        let number_of_pages = descriptor.page_count;
        let attribute: u64 = match descriptor.att {
            MemoryAttribute::UNCACHEABLE => 0x1,
            MemoryAttribute::WRITE_COMBINE => 0x2,
            MemoryAttribute::WRITE_THROUGH => 0x4,
            MemoryAttribute::WRITE_BACK => 0x8,
            MemoryAttribute::UNCACHABLE_EXPORTED => 0x10,
            MemoryAttribute::WRITE_PROTECT => 0x1000,
            MemoryAttribute::READ_PROTECT => 0x2000,
            MemoryAttribute::EXECUTE_PROTECT => 0x4000,
            MemoryAttribute::NON_VOLATILE => 0x8000,
            MemoryAttribute::MORE_RELIABLE => 0x10000,
            MemoryAttribute::READ_ONLY => 0x20000,
            MemoryAttribute::RUNTIME => 0x8000_0000_0000_0000,
            _ => 0,
        };

        let line = alloc::format!(
            "{:016x},{:016x},{:016x},{:016x}\n",
            memory_type,
            physical_start,
            number_of_pages,
            attribute
        );
        memory_map_file.write(line.as_bytes()).unwrap_success();
    }
    //自分の環境ではこれを書かないと変更が反映されなかった
    memory_map_file.flush().unwrap_success();

    //feature = exts
    let gop_handles = boot_services
        .find_handles::<GraphicsOutput>()
        .unwrap_success();
    //unsafecellなのでget()がいる
    let gop = boot_services
        .handle_protocol::<GraphicsOutput>(gop_handles[0])
        .unwrap_success()
        .get();
    //gopがおかしいとpanicが呼ばれるのでここまで動いていたら安全なはず
    let mut frame_buffer = unsafe { (*gop).frame_buffer() };
    for i in 0..frame_buffer.size() {
        //安全性は不明
        unsafe {
            frame_buffer.write_byte(i, 255);
        }
    }

    let kernel_file_handle = root_dir
        .open("\\kernel.elf", FileMode::Read, FileAttribute::empty())
        .unwrap_success();
    //安全性は不明
    let mut kernel_file = unsafe { RegularFile::new(kernel_file_handle) };
    //バッファのサイズ=構造体のサイズ+ファイル名(kernel.elf=12*16bit)
    let file_info_buffer = &mut [0; 80 + 24];
    let file_info: &mut file::FileInfo = kernel_file.get_info(file_info_buffer).unwrap().unwrap();
    let kernel_file_size = file_info.file_size();
    let page_count = ((kernel_file_size + 0xfff) / 0x1000) as usize;
    //elfからbase_addrを取得する方法がよくわからなかったのでハードコーディング
    let base_addr = 0x200000;
    system_table
        .boot_services()
        .allocate_pages(
            AllocateType::Address(base_addr),
            MemoryType::LOADER_DATA,
            page_count,
        )
        .unwrap_success();
    //配列のサイズが分からないのでallocate_poolから変換
    let file_buffer_addr = boot_services
        .allocate_pool(MemoryType::LOADER_DATA, kernel_file_size as usize)
        .unwrap_success();
    //安全性は不明
    let file_buffer =
        unsafe { slice::from_raw_parts_mut(file_buffer_addr, kernel_file_size as usize) };
    kernel_file.read(file_buffer).unwrap_success();

    let elf_header = ElfHeader::new(file_buffer);
    let entry_point_addr = elf_header.entry as *const ();
    let program_header_iter = ProgramHeaderIter::new(file_buffer);
    for program_header in program_header_iter {
        let addr = program_header.paddr as *mut u8;
        let offset = program_header.offset as usize;
        let size = program_header.memsz;
        //読み込みは1バイト単位
        //もっといい方法があるかも？
        for i in 0..size {
            //安全性は不明
            unsafe {
                boot_services.memset(addr.offset(i as isize), 1, file_buffer[offset + i as usize]);
            }
        }
    }

    //exit
    writeln!(stdout, "Bye").unwrap();
    system_table
        .exit_boot_services(handle, memory_map_buffer)
        .unwrap_success();

    //entry
    //elfの仕様に則っているので安全なはず
    let entry_point = unsafe {
        mem::transmute::<*const (), extern "sysv64" fn(args_ptr: *mut FrameBuffer) -> !>(
            entry_point_addr,
        )
    };
    entry_point(&mut frame_buffer);
}
