# ______________________________________________________________________________
#
#  MRE Makefile
# ______________________________________________________________________________
# 
GCC = arm-none-eabi-gcc
GPP = arm-none-eabi-g++
PYTHON = python3
OBJCOPY = objcopy

SDK = sdk
APP_FILE_NAME = MyMREApp

INCLUDES = -I "$(SDK)/include" -I "$(SDK)/shims/include" -I "$(SDK)/shims/ResID" -I "$(SDK)/shims" 
DEFINES = -D _MINIGUI_LIB_ -D _USE_MINIGUIENTRY -D _NOUNIX_ -D _FOR_WNC -D __MRE_SDK__ -D __MRE_VENUS_NORMAL__ -D __MMI_MAINLCD_240X320__ -D MRE -D GCC -D __MRE_COMPILER_GCC__

SHARED_PARAM = -c -fpic -march=armv5te -fvisibility=hidden -Os -mlittle-endian $(INCLUDES) $(DEFINES) -fdata-sections -ffunction-sections
GCC_PARAM = $(SHARED_PARAM)
GPP_PARAM = $(SHARED_PARAM) -fno-threadsafe-statics -fno-exceptions -fno-non-call-exceptions

LINK_LIB = -lstdc++ $(SDK)/lib/*.a 
LINK_PARAM = -fno-threadsafe-statics -Wl,--gc-sections -fpic -fpcc-struct-return --disable-libstdcxx-verbose -pie -lm -T "$(SDK)/scat.ld"

RUST_LIB = target/armv5te-unknown-linux-gnueabi/release/libmy_mre_app.a

# ______________________________________________________________________________
# 
all: build $(RUST_LIB) $(APP_FILE_NAME)_patched.vxp

$(RUST_LIB):
	export LIBCLANG_PATH=/home/cipl1192/sandbox/llvm/clang+llvm-17.0.6-x86_64-linux-gnu-ubuntu-22.04/lib && cargo build --target armv5te-unknown-linux-gnueabi --release

# 1. Compile
build/%.o: $(SDK)/shims/%.c
	$(GCC) $(GCC_PARAM) -c $< -o $@

build/%.o: $(SDK)/shims/%.cpp
	$(GPP) $(GPP_PARAM) -c $< -o $@

# 2. Link
build/$(APP_FILE_NAME).axf: $(patsubst $(SDK)/shims/%.c,build/%.o,$(wildcard $(SDK)/shims/*.c)) $(RUST_LIB)
	$(GCC) -o $@ $^ $(LINK_LIB) $(LINK_PARAM)

# 3. Add resources
build/$(APP_FILE_NAME).vxp: build/$(APP_FILE_NAME).axf
	$(OBJCOPY) -I elf32-little --add-section .vm_res=$(SDK)/resource.bin $^ $@

# 4. Add tags
$(APP_FILE_NAME).vxp: build/$(APP_FILE_NAME).vxp
	$(PYTHON) $(SDK)/build.py $^ $@

build/vxpatch: $(SDK)/tools/vxpatch.rs | build
	rustc $< -o $@

# 5. Patch VXP using IMSI from imsi.txt
$(APP_FILE_NAME)_patched.vxp: $(APP_FILE_NAME).vxp imsi.txt build/vxpatch
	./build/vxpatch $< $@

build:
	mkdir -p build

clean:
	rm -rf build
	rm -f *.vxp
