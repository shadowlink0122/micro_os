#![no_std]
#![no_main]
#![feature(offset_of)]
use core::fmt::Write;
use core::panic::PanicInfo;
use core::writeln;

use micro_os::error;
use micro_os::graphics::fill_rect;
use micro_os::graphics::Bitmap;
use micro_os::info;
use micro_os::init::init_basic_runtime;
use micro_os::println;
use micro_os::qemu::exit_qemu;
use micro_os::qemu::QemuExitCode;
use micro_os::uefi::init_vram;
use micro_os::uefi::EfiHandle;
use micro_os::uefi::EfiMemoryType;
use micro_os::uefi::EfiSystemTable;
use micro_os::uefi::VramTextWriter;
use micro_os::warn;
use micro_os::x86::hlt;
use micro_os::x86::init_exceptions;
use micro_os::x86::trigger_debug_interrupt;

#[no_mangle]
fn efi_main(image_handle: EfiHandle, efi_system_table: &EfiSystemTable) {
    let mut vram = init_vram(efi_system_table).expect("init_vram failed");
    let vw = vram.width();
    let vh = vram.height();
    fill_rect(&mut vram, 0x0000ff, 0, 0, vw, vh).expect("fill_rect failed");

    let mut w = VramTextWriter::new(&mut vram);

    println!("Booting micro_os...");
    println!("image_handle: {:#018X}", image_handle);
    println!("efi_system_table: {:#p}", efi_system_table);

    let loaded_image_prptocol =
        micro_os::uefi::locate_loaded_image_protocol(image_handle, efi_system_table)
            .expect("locate_loaded_image_protocol failed");
    println!("image_size: {:#018X}", loaded_image_prptocol.image_size);
    println!("image_base: {:#018X}", loaded_image_prptocol.image_base);
    info!("info");
    warn!("warn");
    error!("error");

    let memory_map = init_basic_runtime(image_handle, efi_system_table);
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

    writeln!(w, "Hello, Non-UEFI World!").unwrap();

    let cr3 = micro_os::x86::read_cr3();
    println!("cr3 = {cr3:#p}");
    let t = Some(unsafe { &*cr3 });
    println!("{t:?}");
    let t = t.and_then(|t| t.next_level(0));
    println!("{t:?}");
    let t = t.and_then(|t| t.next_level(0));
    println!("{t:?}");
    let t = t.and_then(|t| t.next_level(0));
    println!("{t:?}");

    let (_gdt, _idt) = init_exceptions();
    info!("Exception initialized!");
    trigger_debug_interrupt();
    info!("Exception continued.");

    loop {
        hlt();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    exit_qemu(QemuExitCode::Fail);
}
