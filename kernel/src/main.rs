#![feature(abi_efiapi)]
#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FrameBuffer{
    base: *mut u8,
    size: usize,
}

impl FrameBuffer{
    pub fn size(&self) -> usize {
        self.size
    }

    pub unsafe fn write_byte(&mut self, index: usize, value: u8) {
        self.base.add(index).write_volatile(value)
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn _start(args_ptr: *mut FrameBuffer) -> ! {

    let mut frame_buffer: FrameBuffer = unsafe { *args_ptr };

    for i in 0..frame_buffer.size() {
        unsafe {
            frame_buffer.write_byte(i, (i % 256) as u8);
        }
    }

    loop {}
}
