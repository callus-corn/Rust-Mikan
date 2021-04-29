#![feature(abi_efiapi)]
#![feature(alloc_error_handler)]
#![no_std]
#![no_main]

extern crate alloc;

use bootloader::arg;
use bootloader::elf::Elf;
use bootloader::vga::Writer;
use core::alloc::Layout;
use core::fmt::Write;
use core::mem;
use core::panic::PanicInfo;
use core::slice;
use uefi::prelude::*;
use uefi::proto::console::gop::{GraphicsOutput, PixelFormat};
use uefi::proto::media::file::{File, FileAttribute, FileInfo, FileMode, RegularFile};
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

    //feature = exts
    let file_system = boot_services
        .get_image_file_system(handle)
        .unwrap_success()
        .get();
    //生ポインタ解決
    //安全性はget_image_file_systemに依存
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
    //安全性はhandle_protocolに依存
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
    //kernel_file_info_bufferのサイズ=構造体のサイズ+ファイル名(kernel.elf=12*16bit)
    let kernel_file_info_buffer = &mut [0; 80 + 24];
    let kernel_file_info: &mut FileInfo  = kernel_file.get_info(kernel_file_info_buffer).unwrap().unwrap();
    let kernel_file_size = kernel_file_info.file_size();
    let kernel_file_buffer_ptr = boot_services.allocate_pool(MemoryType::LOADER_DATA, kernel_file_size as usize).unwrap_success();
    //安全性はallocate_poolに依存
    let kernel_file_buffer = unsafe { slice::from_raw_parts_mut(kernel_file_buffer_ptr, kernel_file_size as usize) };
    kernel_file.read(kernel_file_buffer).unwrap_success();

    let elf_file = Elf::new(kernel_file_buffer);
    let kernel_base_addr = elf_file.calculate_base_addr() as usize;
    let kernel_page_count = elf_file.calculate_page_count();
    boot_services.allocate_pages(AllocateType::Address(kernel_base_addr), MemoryType::LOADER_DATA, kernel_page_count).unwrap_success();
    for program_header in elf_file.program_header_iter() {
        if !program_header.type_is_load() {
            continue;
        }
        let addr = program_header.p_vaddr() as *mut u8;
        let offset = program_header.p_offset() as usize;
        let size = program_header.p_memsz();
        //読み込みは1バイト単位
        //もっといい方法があるかも？
        for i in 0..size {
            //安全性は不明
            unsafe {
                boot_services.memset(addr.offset(i as isize), 1, kernel_file_buffer[offset + i as usize]);
            }
        }
    }

    writeln!(stdout, "Bye").unwrap();
    system_table
        .exit_boot_services(handle, memory_map_buffer)
        .unwrap_success();

    let kernel_entry_point = elf_file.entry() as *const ();
    let kernel_entry = unsafe {
        mem::transmute::<*const (), extern "sysv64" fn(args_ptr: *const arg::Argument) -> !>(
            kernel_entry_point,
        )
    };
    let frame_buffer_base = frame_buffer.as_mut_ptr();
    let frame_buffer_size = frame_buffer.size();
    let arg_frame_buffer = arg::FrameBuffer {
        base: frame_buffer_base,
        size: frame_buffer_size,
    };
    //ここまで動いてるなら安全
    let gop_mode_info = unsafe { (*gop).current_mode_info() };
    let pixels_per_scan_line = gop_mode_info.stride();
    let (horizontal_resolution, vertical_resolution) = gop_mode_info.resolution();
    let pixel_format = match gop_mode_info.pixel_format() {
        PixelFormat::Rgb => arg::PixelFormat::Rgb,
        PixelFormat::Bgr => arg::PixelFormat::Bgr,
        _ => panic!("Unimplemented"),
    };
    let arg_frame_buffer_config = arg::FrameBufferConfig {
        pixels_per_scan_line: pixels_per_scan_line,
        horizontal_resolution: horizontal_resolution,
        vertical_resolution: vertical_resolution,
        pixel_format: pixel_format,
    };
    let args = arg::Argument {
        frame_buffer: arg_frame_buffer,
        frame_buffer_config: arg_frame_buffer_config,
    };
    kernel_entry(&args);
}
