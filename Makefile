ARTIFACTS_FOLDER := artifacts
LIBQ_SRC_FILES=$(shell find src/libq -name '*.c')
SRC_FILES=$(shell find src -type f -name '*.c' -not -path 'src/libq/*')
BINS=$(addprefix $(ARTIFACTS_FOLDER)/, $(SRC_FILES:.c=))
CC=gcc
CC_ARGS=-I lib -Wall -Wextra -Werror
LIBS=/lib64/libc.so.6 /lib64/ld-linux-x86-64.so.2 /lib64/libtinfo.so.6 /lib64/libdl.so.2
INIT_RAM_FS=$(ARTIFACTS_FOLDER)/initramfs.cpio.gz

all: $(BINS) $(INIT_RAM_FS)

$(ARTIFACTS_FOLDER)/src/%: src/%.c
	mkdir -p $(dir $@)
	$(CC) $(CC_ARGS) $(LIBQ_SRC_FILES) $< -o $@

$(INIT_RAM_FS): $(BINS)
	mkdir -p $(dir $@)/initramfs/{bin,lib64}
	cp $(BINS) $(dir $@)/initramfs/bin
	cp /bin/sh $(dir $@)/initramfs/bin
	cp $(LIBS) $(ARTIFACTS_FOLDER)/initramfs/lib64
	cp src/init.sh $(dir $@)/initramfs/init
	chmod +x $(dir $@)/initramfs/init
	find ./artifacts/initramfs -print0 | sed 's;/artifacts/initramfs;;g' | cpio -D ./artifacts/initramfs --null --create --verbose --format=newc | gzip --best > $@

.PHONY: run
run: $(INIT_RAM_FS)
	qemu-system-x86_64 -kernel bin/vmlinux -initrd $(INIT_RAM_FS) -serial stdio -append "root=sr0 console=ttyAMA0  console=ttyS0"

.PHONY: clean
clean:
	rm -rf artifacts