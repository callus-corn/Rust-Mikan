#![feature(abi_efiapi)]
#![feature(alloc_error_handler)]
#![no_std]
#![no_main]

use uefi::prelude::*;
use uefi::table::boot::{MemoryType, MemoryAttribute, AllocateType};
use uefi::proto::loaded_image::LoadedImage;
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::proto::media::file;
use uefi::proto::media::file::{File, RegularFile, Directory, FileMode, FileAttribute};
use uefi::proto::console::gop::{GraphicsOutput, FrameBuffer};
use core::mem;
use core::alloc::Layout;
use core::panic::PanicInfo;
use core::fmt::Write;


#[alloc_error_handler]
fn on_oom(_layout: Layout) -> ! {
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

//数値→ascii列変換
fn u32_to_ascii(number: u32) -> [u8;8] {
    let mut result: [u8;8] = [0;8];
    let radix = 16;
    let len = result.len();
    for i in 0..len {
        let target_4bit = ((number >> i*4) % radix) as u8;
        if target_4bit <= 0x9 {
            result[i] = 0x30 + target_4bit;
        } else if target_4bit >= 0xa && target_4bit <= 0xf {
            result[i] = 0x57 + target_4bit;
        }
    }
    result
}

//数値→ascii列変換
fn u64_to_ascii(number: u64) -> [u8;16] {
    let mut result: [u8;16] = [0;16];
    let radix = 16;
    let len = result.len();
    for i in 0..len {
        let target_4bit = ((number >> i*4) % radix) as u8;
        if target_4bit <= 0x9 {
            result[i] = 0x30 + target_4bit;
        } else if target_4bit >= 0xa && target_4bit <= 0xf {
            result[i] = 0x57 + target_4bit;
        }
    }
    result
}

#[entry]
fn efi_main(handle: Handle, system_table: SystemTable<Boot>) -> Status {

    unsafe{
        uefi::alloc::init(system_table.boot_services());
    }

    writeln!(system_table.stdout(), "Hello, world!").unwrap();

    //↓メモリマップの取得
    //メモリマップを書き込むバッファ（サイズは適当）
    let memory_map_buffer: &mut [u8] = &mut [0; 4096*4];
    //帰ってくるのはmap_keyとdescriptorのイテレータ（イテレータの中にメモリマップがある）
    //このResultはuefi-rs独自実装のためunwrap_successを使う。
    let (_memory_map_key, descriptor_iter) = system_table.boot_services().memory_map(memory_map_buffer).unwrap_success();
    //↑メモリマップの取得
    
    //↓ルートディレクトリを開く
    //ほしいプロトコルを指定してHandleを渡す。帰ってくるのはUnsafeCell<プロトコル>なのでgetで中身を取り出す
    let loaded_image = system_table.boot_services().handle_protocol::<LoadedImage>(handle).unwrap_success().get();
    //生ポインタを解決するのでunsafe
    let device;
    unsafe {
        device = (*loaded_image).device();
    }
    let file_system = system_table.boot_services().handle_protocol::<SimpleFileSystem>(device).unwrap_success().get();
    //再度生ポインタ
    let mut root_dir: Directory;
    unsafe {
        root_dir = (*file_system).open_volume().unwrap_success();
    }
    //↑ルートディレクトリを開く

    //↓メモリマップの保存
    //保存するファイルの作成とFileHandleの取得
    let memory_map_file_handle = root_dir.open("\\memmap",FileMode::CreateReadWrite,FileAttribute::empty()).unwrap_success();
    //RegularFileに変換する必要あり(unsafe)
    let mut memory_map_file: RegularFile;
    unsafe {
        memory_map_file = RegularFile::new(memory_map_file_handle);
    }
    //ヘッダの書き込み
    let header: &[u8] = "Type, PhysicalStart, NumberOfPages, Attribute\n".as_bytes();
    memory_map_file.write(header).unwrap_success();
    //メモリディスクリプタの書き込み
    for descriptor in descriptor_iter {
        let memory_type:u32 = match descriptor.ty {
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

        //上手く変換できなかったのでゴリ押し
        //絶対にもっといい方法がある
        let buffer: &mut [u8] = &mut [0;63];
        let memory_type = u32_to_ascii(memory_type);
        let physical_start = u64_to_ascii(physical_start);
        let number_of_pages = u64_to_ascii(number_of_pages);
        let attribute = u64_to_ascii(attribute);

        //memory_typeゴリ押し
        let memory_type_len = memory_type.len();
        //下駄。paddingといっていいんだろうか？
        let padding = 0;
        for i in 0..memory_type_len {
            buffer[padding+i] = memory_type[memory_type_len-i-1];
        }
        buffer[padding+memory_type_len] = 0x2c;//,
        buffer[padding+memory_type_len+1] = 0x20;//空白

        //physical_startゴリ押し
        let physical_start_len = physical_start.len();
        let padding = memory_type_len + 2;
        for i in 0..physical_start_len {
            buffer[padding+i] = physical_start[physical_start_len-i-1];
        }
        buffer[padding+physical_start_len] = 0x2c;//,
        buffer[padding+physical_start_len+1] = 0x20;//空白

        //number_of_pagesゴリ押し
        let number_of_pages_len = number_of_pages.len();
        let padding = memory_type_len + 2 + physical_start_len + 2;
        for i in 0..number_of_pages_len {
            buffer[padding + i] = number_of_pages[number_of_pages_len-i-1];
        }
        buffer[padding+number_of_pages_len] = 0x2c;//,
        buffer[padding+number_of_pages_len+1] = 0x20;//空白

        //attributeゴリ押し
        let attribute_len = attribute.len();
        let padding = memory_type_len + 2 + physical_start_len + 2 + number_of_pages_len + 2;
        for i in 0..attribute_len {
            buffer[padding+i] = attribute[attribute_len-i-1];
        }
        buffer[padding+attribute_len] = 0x0a;//LF

        memory_map_file.write(buffer).unwrap_success();
    }
    //書き込みの反映。自分の環境ではこれを書かないと変更が反映されなかった
    memory_map_file.flush().unwrap_success();
    //↑メモリマップの保存

    //↓openGOP
    //LocateHandleBuffer(gEfiGraphicsOutputProtocolGuid)
    writeln!(system_table.stdout(), "Hello, world!").unwrap();
    let gop_handles = system_table.boot_services().find_handles::<GraphicsOutput>().unwrap_success();
    //OpenProtocol
    writeln!(system_table.stdout(), "Hello, world!").unwrap();
    let gop = system_table.boot_services().handle_protocol::<GraphicsOutput>(gop_handles[0]).unwrap_success().get();
    //↑openGOP

    //frame_buffer書き込み
    writeln!(system_table.stdout(), "Hello, world!").unwrap();
    let mut frame_buffer: FrameBuffer;
    unsafe {
        frame_buffer = (*gop).frame_buffer();
    }
    writeln!(system_table.stdout(), "Hello, world!").unwrap();
    for i in 0..frame_buffer.size() {
        unsafe {
            frame_buffer.write_byte(i,255);
        }
    }

    //open kernel.elf
    writeln!(system_table.stdout(), "Hello, world!").unwrap();
    let kernel_file_handle = root_dir.open("\\kernel.elf",FileMode::Read,FileAttribute::empty()).unwrap_success();
    let mut kernel_file: RegularFile;
    unsafe {
        kernel_file = RegularFile::new(kernel_file_handle);
    }
    writeln!(system_table.stdout(), "Hello, world!").unwrap();
    //info取得
    let file_info_buffer: &mut [u8] = &mut [0; 80+24];
    let file_info: &mut file::FileInfo = kernel_file.get_info(file_info_buffer).unwrap().unwrap();
    writeln!(system_table.stdout(), "Hello, world!").unwrap();
    //サイズ取得
    let kernel_file_size = file_info.file_size();
    let page_count = ((kernel_file_size + 0xfff) / 0x1000) as usize;
    //execのために仕方なく
    let page_count = page_count*2;
    //AllocatePages
    let base_addr = 0x200000;
    system_table.boot_services().allocate_pages(AllocateType::Address(base_addr), MemoryType::LOADER_DATA, page_count).unwrap_success();
    writeln!(system_table.stdout(), "Hello, world!").unwrap();
    //read
    let file_buffer: &mut [u8] = &mut [0; 0x2000];
    kernel_file.read(file_buffer).unwrap_success();
    for i in 0..file_buffer.len() {
        let addr = base_addr as *mut u8;
        unsafe {
            system_table.boot_services().memset(addr.offset(i as isize), 1, file_buffer[i]);
        }
    }
    writeln!(system_table.stdout(), "Hello, world!").unwrap();
    let exec_addr = base_addr+0x1000;
    //entrypointの調整
    for i in 0..file_buffer.len() {
        let addr = exec_addr as *mut u8;
        unsafe {
            system_table.boot_services().memset(addr.offset(i as isize), 1, file_buffer[i]);
        }
    }
    writeln!(system_table.stdout(), "Bye").unwrap();
    //exit
    system_table.exit_boot_services(handle, memory_map_buffer).unwrap_success();
    //entry
    unsafe {
        let entry_point = mem::transmute::<*const (), extern "sysv64" fn(args_ptr: *mut FrameBuffer) -> !>(0x201120 as *const ());
        entry_point(&mut frame_buffer);
    }

    //writeln!(system_table.stdout(), "Kernel did not execute").unwrap();

    //loop {}
    //Status::SUCCESS
}