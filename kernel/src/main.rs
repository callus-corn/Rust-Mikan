#![feature(abi_efiapi)]
#![no_std]
#![no_main]

use core::fmt::Write;
use core::panic::PanicInfo;
use kernel::arg::{Argument, PixelFormat};
use kernel::console::ConsoleWriter;
use kernel::font::FontWriter;
use kernel::graphic::{PixelColor, PixelWriter};
use kernel::pci::{Configuration, Pci};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn _start(args_ptr: *const Argument) -> ! {
    let args = unsafe { *args_ptr };
    let frame_buffer = args.frame_buffer;
    let frame_buffer_config = args.frame_buffer_config;
    let pixel_writer = match frame_buffer_config.pixel_format {
        PixelFormat::Rgb => PixelWriter::new_rgb(frame_buffer, frame_buffer_config),
        PixelFormat::Bgr => PixelWriter::new_bgr(frame_buffer, frame_buffer_config),
    };

    for x in 0..pixel_writer.horizontal_resolution() {
        for y in 0..pixel_writer.vertical_resolution() {
            //一応エラー処理、エラーはめんどうなので無視
            match pixel_writer.write(x, y, PixelColor::BACKGROUND) {
                Ok(_) => (),
                Err(_) => (),
            };
        }
    }

    //Consoleの依存をFontに集約したかったのでFontWriterを追加
    let font_writer = FontWriter::new(pixel_writer);
    let mut console_writer = ConsoleWriter::new(font_writer);
    write!(console_writer, "Welcome to MikanOS\n").unwrap();

    let pci = Pci::new();
    for device in pci.iter() {
        let vender_id = Configuration::vender_id(&device);
        let base_class = Configuration::base_class(&device);
        let sub_class = Configuration::sub_class(&device);
        let header_type = Configuration::header_type(&device);
        write!(
            console_writer,
            "{}.{}.{}: vneder {:x}, base class {}, sub class {}, head {}\n",
            device.bus(),
            device.device(),
            device.function(),
            vender_id,
            base_class,
            sub_class,
            header_type
        )
        .unwrap();
    }

    loop {}
}
