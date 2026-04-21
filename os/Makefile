TARGET := riscv64gc-unknown-none-elf
MODE := release
KERNEL_ELF := target/$(TARGET)/$(MODE)/new_rcore
KERNEL_BIN := $(KERNEL_ELF).bin

# BOARD
BOARD := qemu
SBI ?= rustsbi
# BOOTLOADER := ../bootloader/$(SBI)-$(BOARD).bin
BOOTLOADER := ../bootloader/$(SBI)-$(BOARD)-0.4.0.bin

etest: 
	@echo "Testing..."
	@echo "target: $(TARGET)"
	@echo "mode: $(MODE)"
	@echo "kernel elf: $(KERNEL_ELF)"
	@echo "kernel bin: $(KERNEL_BIN)"
	@echo "bootloader: $(BOOTLOADER)"

build:
	@cargo build --target $(TARGET) --$(MODE)


# KERNEL ENTRY
KERNEL_ENTRY_PA := 0x80200000

# Binutils
OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64

# Disassembly
DISASM ?= -x

$(KERNEL_BIN): build
	@echo "$(OBJCOPY) --strip-all $(KERNEL_ELF) -O binary $@"
	@$(OBJCOPY) --strip-all $(KERNEL_ELF) -O binary $@

clean:
	@cargo clean

run: build $(KERNEL_BIN)
	@qemu-system-riscv64 \
		-machine virt \
		-nographic \
		-bios $(BOOTLOADER) \
		-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA)

debug-run: build $(KERNEL_BIN)
	@qemu-system-riscv64 \
		-machine virt \
		-nographic \
		-bios $(BOOTLOADER) \
		-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA) \
		-s -S

debug:
	@gdb \
		-ex "file $(KERNEL_ELF)" \
		-ex "set arch riscv:rv64" \
		-ex "target remote localhost:1234"

.PHONY: etest build clean run debug-run debug