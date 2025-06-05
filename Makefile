EFIDIR=mnt/EFI/BOOT
EFI=${EFIDIR}/BOOTX64.EFI
TARGET=x86_64-unknown-uefi

default: all
init:
	mkdir -p ${EFIDIR}

build:
	cargo build --target ${TARGET}

cp_binary:
	cp target/${TARGET}/debug/micro_os.efi ${EFI}

run:
	qemu-system-x86_64 -bios third-party/ovmf/RELEASEX64_OVMF.fd -drive format=raw,file=fat:rw:mnt

# build, copy, run
all:
	cargo build --target ${TARGET}
	cp target/${TARGET}/debug/micro_os.efi ${EFI}
	qemu-system-x86_64 -bios third-party/ovmf/RELEASEX64_OVMF.fd -drive format=raw,file=fat:rw:mnt
