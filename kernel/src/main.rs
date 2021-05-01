#![feature(abi_efiapi)]
#![no_std]
#![no_main]

use core::panic::PanicInfo;
use kernel::arg::{Argument, PixelFormat};
use kernel::font::Ascii;
use kernel::graphic::{PixelColor, Writer};

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
        PixelFormat::Rgb => Writer::new_rgb(frame_buffer, frame_buffer_config),
        PixelFormat::Bgr => Writer::new_bgr(frame_buffer, frame_buffer_config),
    };

    for x in 0..pixel_writer.horizontal_resolution() {
        for y in 0..pixel_writer.vertical_resolution() {
            let white = PixelColor {
                r: 255,
                g: 255,
                b: 255,
            };
            //一応エラー処理、エラーはめんどうなので無視
            match pixel_writer.write(x, y, white) {
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

    let ascii = Ascii::new(pixel_writer);
    ascii.write("1 + 2 = ");

    loop {}
}
