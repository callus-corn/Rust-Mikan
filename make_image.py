import subprocess as proc

efi_path = "bootloader/target/x86_64-unknown-uefi/release/bootloader.efi"
proc.call('cp '+ efi_path +' BOOTX64.EFI', shell=True)

kernel_path = "kernel/target/x86_64/release/kernel.elf"
proc.call('cp '+ kernel_path +' kernel.elf', shell=True)

proc.call('qemu-img create -f raw disk.img 200M', shell=True)
proc.call('mkfs.fat -n \'MIKAN OS\' -s 2 -f 2 -R 32 -F 32 disk.img', shell=True)
proc.call('mkdir -p mnt', shell=True)
proc.call('mount -o loop disk.img mnt', shell=True)
proc.call('mkdir -p mnt/EFI/BOOT', shell=True)
proc.call('cp BOOTX64.EFI mnt/EFI/BOOT/BOOTX64.EFI', shell=True)
proc.call('cp kernel.elf mnt/kernel.elf', shell=True)
proc.call('umount mnt', shell=True)

proc.call('rm -rf mnt', shell=True)
proc.call('rm BOOTX64.EFI', shell=True)
proc.call('rm kernel.elf', shell=True)
