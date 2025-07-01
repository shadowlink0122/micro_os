COMPILE_IMAGE=micro_os.efi
EFIDIR=mnt/EFI/BOOT
EFI=${EFIDIR}/BOOTX64.EFI
TARGET=x86_64-unknown-uefi

default: rust run
init:
	mkdir -p ${EFI}

clean:
	rm -rf ${EFI} cpp/${COMPILE_IMAGE} cpp/main.o

# build rust code
.PHONY: rust
rust:
	cargo build --target ${TARGET}
	rm -rf ${EFI}
	cp target/${TARGET}/debug/${COMPILE_IMAGE} ${EFI}

# build cpp code
.PHONY: cpp
cpp:
	cd cpp && clang -target x86_64-pc-win32-coff -mno-red-zone -fno-stack-protector \
		-fshort-wchar -Wall -c main.cpp
	cd cpp && lld-link /subsystem:efi_application /entry:EfiMain /out:${COMPILE_IMAGE} main.o
	rm -rf ${EFI}
	cp cpp/${COMPILE_IMAGE} ${EFI}

.PHONY: test
test: test_rust

.PHONY: test_rust
test_rust:
	cargo test

test_cpp:
	:

# run efi image on qemu
run:
	qemu-system-x86_64 -m 2G -bios third-party/ovmf/RELEASEX64_OVMF.fd \
    -drive format=raw,file=fat:rw:mnt \
    -chardev stdio,id=char_com1,mux=on,logfile=log/com1.txt \
    -serial chardev:char_com1 \
    -device isa-debug-exit,iobase=0xf4,iosize=0x01
