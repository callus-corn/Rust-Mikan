#![feature(abi_efiapi)]
#![no_std]
#![no_main]

use kernel::arg::{Argument, PixelFormat};
use kernel::console::ConsoleWriter;
use kernel::graphic::{PixelColor, PixelWriter};
use core::fmt::Write;
use core::panic::PanicInfo;

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
            let bg_color = PixelColor::BACKGROUND;
            //一応エラー処理、エラーはめんどうなので無視
            match pixel_writer.write(x, y, bg_color) {
                Ok(_) => (),
                Err(_) => (),
            };
        }
    }

    for x in 0..200 {
        for y in 0..100 {
            let green = PixelColor { r: 0, g: 255, b: 0 };
            //一応エラー処理、エラーはめんどうなので無視
            match pixel_writer.write(x, y, green) {
                Ok(_) => (),
                Err(_) => (),
            };
        }
    }

    let mut console_writer = ConsoleWriter::new(pixel_writer);
    for i in 0..30 {
        write!(console_writer, "console:{}\n", i).unwrap();
    }

    loop {}
}
