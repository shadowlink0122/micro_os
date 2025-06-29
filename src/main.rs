#![no_std]
#![no_main]
#![feature(offset_of)]

use core::fmt::Write;
use core::panic::PanicInfo;
use core::writeln;

use micro_os::graphics::fill_rect;
use micro_os::graphics::Bitmap;
use micro_os::qemu::exit_qemu;
use micro_os::qemu::QemuExitCode;
use micro_os::uefi::exit_from_efi_services;
use micro_os::uefi::init_vram;
use micro_os::uefi::EfiHandle;
use micro_os::uefi::EfiMemoryType;
use micro_os::uefi::EfiSystemTable;
use micro_os::uefi::MemoryMapHolder;
use micro_os::uefi::VramTextWriter;
use micro_os::x86::hlt;

#[no_mangle]
fn efi_main(image_handle: EfiHandle, efi_system_table: &EfiSystemTable) {
    let mut vram = init_vram(efi_system_table).expect("init_vra, failed");

    let vw = vram.width();
    let vh = vram.height();
    fill_rect(&mut vram, 0x0000ff, 0, 0, vw, vh).expect("fill_rect failed");

    let mut w = VramTextWriter::new(&mut vram);
    let mut memory_map = MemoryMapHolder::new();

    let status = efi_system_table
        .boot_servicies()
        .get_memory_map(&mut memory_map);
    writeln!(w, "{status:?}").unwrap();
    let mut total_memory_pages = 0;
    for e in memory_map.iter() {
        if e.memory_type() != EfiMemoryType::CONVENTIONAL_MEMORY {
            continue;
        }
        total_memory_pages += e.number_of_pages();
        writeln!(w, "{e:?}").unwrap();
    }
    let total_memory_size_mib = total_memory_pages * 4096 / 1024 / 1024;
    writeln!(
        w,
        "Total: {total_memory_pages} pages = {total_memory_size_mib}MiB"
    )
    .unwrap();

    exit_from_efi_services(image_handle, efi_system_table, &mut memory_map);
    writeln!(w, "Hello, Non-UEFI World!").unwrap();
    loop {
        hlt();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    exit_qemu(QemuExitCode::Fail);
}
