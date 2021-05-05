use core::iter::Iterator;

#[derive(Copy, Clone)]
pub struct Pci {
    devices: [Device; Pci::CAPACITY],
    size: usize,
}

impl Pci {
    const CAPACITY: usize = 256;//本当は65536だけどスタックが枯渇する
    const MAX_BUS: usize = 256;
    const MAX_DEVICE: usize = 32;
    const MAX_FUNCTION: usize = 8;

    pub fn new() -> Pci {
        let device = Device::new(0,0,0);
        let mut devices = [device; Pci::CAPACITY];
        let mut size = 0;

        //ブリッジが見つかると探索するバスが追加される
        //バス0は必ず探索する
        let mut bus_scan_plan = [0; Pci::MAX_BUS as usize];
        let mut bus_size = 1;
        for i in 0..Pci::MAX_BUS {
            if i >= bus_size {
                //見つかっているバスを全て探索したら終了
                break;
            }
            let bus = bus_scan_plan[i as usize];
            for device in 0..Pci::MAX_DEVICE {
                let new_device = Device::new(bus, device as u8, 0);
                if Configuration::vender_id(&new_device) == 0xffff {
                    //デバイスがない場合
                    continue;
                }
                devices[size] = new_device;
                size += 1;
                if new_device.is_pci_pci_bridge() && bus_size < Pci::MAX_BUS {
                    bus_scan_plan[bus_size as usize] = new_device.secondary_bus();
                    bus_size += 1;
                }
                if new_device.is_single_function() {
                    //一つしかファンクションがない場合
                    continue;
                }
                for function in 1..Pci::MAX_FUNCTION {
                    let new_device = Device::new(bus, device as u8, function as u8);
                    if Configuration::vender_id(&new_device) == 0xffff {
                        //ファンクションがない場合
                        continue;
                    }
                    devices[size] = new_device;
                    size += 1;
                    if new_device.is_pci_pci_bridge() && bus_size < Pci::MAX_BUS {
                        bus_scan_plan[bus_size as usize] = new_device.secondary_bus();
                        bus_size += 1;
                    }
                }
            }
        }

        Pci {
            devices: devices,
            size: size,
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn get(&self, index: usize) -> Option<Device> {
        if index >= self.size {
            ()
        }
        //sizeがおかしくなければunwrapできる
        //参照を扱いたくないので参照外し
        Some(*(self.devices.get(index).unwrap()))
    }

    pub fn iter(&self) -> PciIterator {
        PciIterator::new(*self)
    }
}

pub struct PciIterator {
    pci: Pci,
    count: usize,
}

impl PciIterator {
    pub fn new(pci: Pci) -> PciIterator {
        PciIterator {
            pci: pci,
            count: 0,
        }
    }
}

impl Iterator for PciIterator{
    type Item = Device;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count < self.pci.size() {
            let device = self.pci.get(self.count);
            self.count += 1;
            device
        } else {
            None
        }
    }
}

#[derive(Copy, Clone)]
pub struct Device {
    bus: u8,
    device: u8,
    function: u8,
}

impl Device {
    pub fn new(bus: u8, device: u8, function: u8) -> Device {
        Device {
            bus,
            device,
            function,
        }
    }

    pub fn bus(&self) -> u8 {
        self.bus
    }

    pub fn device(&self) -> u8 {
        self.device
    }

    pub fn function(&self) -> u8 {
        self.function
    }

    pub fn is_single_function(&self) -> bool {
        Configuration::header_type(self) & 0x80 == 0
    }

    pub fn is_pci_pci_bridge(&self) -> bool {
        Configuration::base_class(self) == 0x06 && Configuration::sub_class(self) == 0x04
    }

    pub fn secondary_bus(&self) -> u8 {
        ((Configuration::base_address_register_2(self) >> 8) & 0xff) as u8
    }
}

pub struct Configuration {}

impl Configuration {
    const CONFIG_ADDRESS: u16 = 0x0cf8;
    const CONFIG_DATA: u16 = 0x0cfc;

    pub fn vender_id(device: &Device) -> u16 {
        let address = Configuration::address(device, 0x00);
        (Configuration::read(address) & 0xffff) as u16
    }

    pub fn header_type(device: &Device) -> u8 {
        let address = Configuration::address(device, 0x0c);
        ((Configuration::read(address) >> 16 ) & 0xff) as u8
    }

    pub fn base_class(device: &Device) -> u8 {
        let address = Configuration::address(device, 0x08);
        ((Configuration::read(address) >> 24) & 0xff) as u8
    }

    pub fn sub_class(device: &Device) -> u8 {
        let address = Configuration::address(device, 0x08);
        ((Configuration::read(address) >> 16) & 0xff) as u8
    }

    pub fn base_address_register_2(device: &Device) -> u32 {
        let address = Configuration::address(device, 0x18);
        Configuration::read(address)
    }

    fn address(device: &Device, register_offset: u8) -> u32 {
        (1 << 31) | ((device.bus() as u32) << 16) | ((device.device() as u32) << 11) | ((device.function() as u32) << 8) | ((register_offset as u32) & 0xfc)
    }

    fn read(data: u32) -> u32 {
        let mut out = 0;
        unsafe {
            asm!("out dx, eax", in("dx") Configuration::CONFIG_ADDRESS, in("eax") data);
            asm!("in eax, dx", inout("eax") out, in("dx") Configuration::CONFIG_DATA);
        }
        out
    }
}