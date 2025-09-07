#![no_std]
#![no_main]
#![feature(offset_of)]
extern crate alloc;
use core::panic::PanicInfo;
use core::time::Duration;
use micro_os::error;
use micro_os::executor::Executor;
use micro_os::executor::Task;
use micro_os::executor::TimeoutFuture;
use micro_os::hpet::global_timestamp;
use micro_os::info;
use micro_os::init::init_allocator;
use micro_os::init::init_basic_runtime;
use micro_os::init::init_display;
use micro_os::init::init_hpet;
use micro_os::init::init_paging;
use micro_os::print::hexdump;
use micro_os::print::set_global_vram;
use micro_os::println;
use micro_os::qemu::exit_qemu;
use micro_os::qemu::QemuExitCode;
use micro_os::serial::SerialPort;
use micro_os::uefi::init_vram;
use micro_os::uefi::locate_loaded_image_protocol;
use micro_os::uefi::EfiHandle;
use micro_os::uefi::EfiSystemTable;
use micro_os::x86::init_exceptions;

#[no_mangle]
fn efi_main(image_handle: EfiHandle, efi_system_table: &EfiSystemTable) {
    hexdump(efi_system_table);
    let mut vram = init_vram(efi_system_table).expect("init_vram failed");
    init_display(&mut vram);
    set_global_vram(vram);

    let acpi = efi_system_table
        .acpi_table()
        .expect("Failed to get ACPI table");

    println!("Booting micro_os...");
    println!("image_handle: {:#018X}", image_handle);
    println!("efi_system_table: {:#p}", efi_system_table);

    let loaded_image_prptocol =
        micro_os::uefi::locate_loaded_image_protocol(image_handle, efi_system_table)
            .expect("locate_loaded_image_protocol failed");
    println!("image_size: {:#018X}", loaded_image_prptocol.image_size);
    println!("image_base: {:#018X}", loaded_image_prptocol.image_base);

    let memory_map = init_basic_runtime(image_handle, efi_system_table);
    info!("Hello, Non-UEFI World!");
    init_allocator(&memory_map);
    let (_gdt, _idt) = init_exceptions();
    init_paging(&memory_map);
    init_hpet(acpi);
    let t0 = global_timestamp();

    let task1 = Task::new(async move {
        for i in 100..=103 {
            info!("{i} hpet.main_counter(): {:?}", global_timestamp() - t0);
            TimeoutFuture::new(Duration::from_secs(1)).await;
        }
        Ok(())
    });

    let task2 = Task::new(async move {
        for i in 200..=203 {
            info!("{i} hpet.main_counter(): {:?}", global_timestamp() - t0);
            TimeoutFuture::new(Duration::from_secs(2)).await;
        }
        Ok(())
    });

    let task3 = Task::new(async move {
        for i in 300..=303 {
            info!("{i} hpet.main_counter(): {:?}", global_timestamp() - t0);
            TimeoutFuture::new(Duration::from_secs(3)).await;
        }
        Ok(())
    });

    let serial_task = Task::new(async {
        let sp = SerialPort::default();
        if let Err(e) = sp.loopback_test() {
            error!("{e:?}");
            return Err("serial: loopback test failed");
        }
        info!("Started to minitor serial port");
        loop {
            if let Some(v) = sp.try_read() {
                let c = core::char::from_u32(v as u32);
                info!("serial input: {v:#04X} = {c:?}");
            }
            TimeoutFuture::new(Duration::from_millis(20)).await;
        }
    });

    let mut executor = Executor::new();
    executor.enqueue(task1);
    executor.enqueue(task2);
    executor.enqueue(task3);
    executor.enqueue(serial_task);
    Executor::run(executor)

    // loop {
    //     hlt();
    // }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    exit_qemu(QemuExitCode::Fail);
}
