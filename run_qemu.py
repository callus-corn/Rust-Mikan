import subprocess as proc

proc.call('qemu-system-x86_64 -m 512M -drive if=pflash,format=raw,readonly,file=./OVMF_CODE.fd -drive if=pflash,format=raw,file=./OVMF_VARS.fd -drive if=ide,index=0,media=disk,format=raw,file=disk.img -device nec-usb-xhci,id=xhci -device usb-mouse -device usb-kbd -monitor stdio', shell=True)
