#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use core::panic::PanicInfo;

// Include the generated bindings
pub mod sdk {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]
    include!("mre_bindings.rs");
}
pub use sdk::*;

// Some constants from test.c
const COLOR_BG: VMUINT16 = VM_COLOR_BLACK as u16;
const COLOR_TEXT: VMUINT16 = VM_COLOR_WHITE as u16;
const COLOR_SEL: VMUINT16 = VM_COLOR_GREEN as u16;
const COLOR_BAR: VMUINT16 = 0xA554;

#[derive(Clone, Copy, PartialEq)]
enum Page {
    Menu,
    Ram,
    Screen,
}

static mut LAYER: VMINT = -1;
static mut UCS2_BUF: [VMWCHAR; 128] = [0; 128];
static mut CURRENT_PAGE: Page = Page::Menu;
static mut CURSOR: VMINT = 0;

const MENU_ITEMS: [&str; 2] = ["RAM Test\0", "Screen\0"];

// timeval is provided by the bindings

// Define panic handler for no_std
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// We rely on bindgen's sprintf definition

unsafe fn text(x: i32, y: i32, ascii: *const core::ffi::c_char, color: VMUINT16) {
    let mut c: vm_graphic_color = core::mem::zeroed();
    c.vm_color_565 = color as _;
    
    vm_ascii_to_ucs2(
        UCS2_BUF.as_mut_ptr(),
        256,
        ascii as *mut core::ffi::c_char,
    );
    
    vm_graphic_setcolor(&mut c);
    
    vm_graphic_textout_to_layer(
        LAYER,
        x,
        y,
        UCS2_BUF.as_mut_ptr(),
        vm_graphic_get_screen_width(),
    );
}

unsafe fn clear() {
    let mut c: vm_graphic_color = core::mem::zeroed();
    c.vm_color_565 = COLOR_BG as _;
    vm_graphic_setcolor(&mut c);
    
    vm_graphic_fill_rect_ex(
        LAYER,
        0,
        0,
        vm_graphic_get_screen_width(),
        vm_graphic_get_screen_height(),
    );
}

unsafe fn footer(left: *const core::ffi::c_char, right: *const core::ffi::c_char) {
    let h = vm_graphic_get_character_height() + 4;
    let y = vm_graphic_get_screen_height() - h;

    let mut c: vm_graphic_color = core::mem::zeroed();
    c.vm_color_565 = 0x1082;
    vm_graphic_setcolor(&mut c);

    vm_graphic_fill_rect_ex(
        LAYER,
        0,
        y,
        vm_graphic_get_screen_width(),
        h,
    );

    text(2, y + 2, left, VM_COLOR_WHITE as u16);

    vm_ascii_to_ucs2(
        UCS2_BUF.as_mut_ptr(),
        256,
        right as *mut core::ffi::c_char,
    );

    let mut c2: vm_graphic_color = core::mem::zeroed();
    c2.vm_color_565 = VM_COLOR_WHITE as _;
    vm_graphic_setcolor(&mut c2);

    let w = vm_graphic_get_string_width(UCS2_BUF.as_mut_ptr());
    vm_graphic_textout_to_layer(
        LAYER,
        vm_graphic_get_screen_width() - w - 2,
        y + 2,
        UCS2_BUF.as_mut_ptr(),
        vm_graphic_get_screen_width(),
    );
}

unsafe fn draw_menu() {
    clear();
    text(8, 8, "Diagnostics\0".as_ptr() as *const _, COLOR_SEL);

    let row = vm_graphic_get_character_height() + 8;
    let mut line_buf: [u8; 32] = [0; 32];

    for i in 0..2 {
        if i == CURSOR {
            sprintf(line_buf.as_mut_ptr() as *mut _, "> %s\0".as_ptr() as *const _, MENU_ITEMS[i as usize].as_ptr());
        } else {
            sprintf(line_buf.as_mut_ptr() as *mut _, "  %s\0".as_ptr() as *const _, MENU_ITEMS[i as usize].as_ptr());
        }
        
        let color = if i == CURSOR { COLOR_SEL } else { COLOR_TEXT };
        text(8, 40 + i * row, line_buf.as_ptr() as *const _, color);
    }

    footer("Select\0".as_ptr() as *const _, "Exit\0".as_ptr() as *const _);
    vm_graphic_flush_layer(&mut LAYER as *mut _, 1);
}

unsafe fn draw_ram() {
    clear();
    let m = vm_get_malloc_stat();
    let mut buf: [u8; 64] = [0; 64];

    text(8, 8, "Heap Information\0".as_ptr() as *const _, COLOR_SEL);

    sprintf(buf.as_mut_ptr() as *mut _, "Current : %d\0".as_ptr() as *const _, (*m).current);
    text(8, 40, buf.as_ptr() as *const _, COLOR_TEXT);

    sprintf(buf.as_mut_ptr() as *mut _, "Peak    : %d\0".as_ptr() as *const _, (*m).peak);
    text(8, 60, buf.as_ptr() as *const _, COLOR_TEXT);

    sprintf(buf.as_mut_ptr() as *mut _, "Free    : %d\0".as_ptr() as *const _, (*m).avail_heap_size);
    text(8, 80, buf.as_ptr() as *const _, COLOR_TEXT);

    sprintf(buf.as_mut_ptr() as *mut _, "Mallocs : %d\0".as_ptr() as *const _, (*m).malloc_count);
    text(8, 100, buf.as_ptr() as *const _, COLOR_TEXT);

    sprintf(buf.as_mut_ptr() as *mut _, "Frees   : %d\0".as_ptr() as *const _, (*m).free_count);
    text(8, 120, buf.as_ptr() as *const _, COLOR_TEXT);

    sprintf(buf.as_mut_ptr() as *mut _, "MRE Mem : %u\0".as_ptr() as *const _, vm_get_mre_total_mem_size());
    text(8, 140, buf.as_ptr() as *const _, COLOR_TEXT);

    footer("\0".as_ptr() as *const _, "Back\0".as_ptr() as *const _);
    vm_graphic_flush_layer(&mut LAYER as *mut _, 1);
}

unsafe fn draw_screen() {
    clear();
    let mut buf: [u8; 64] = [0; 64];

    let w = vm_graphic_get_screen_width();
    let h = vm_graphic_get_screen_height();

    text(8, 8, "Screen Information\0".as_ptr() as *const _, COLOR_SEL);

    sprintf(buf.as_mut_ptr() as *mut _, "Width  : %d px\0".as_ptr() as *const _, w);
    text(8, 40, buf.as_ptr() as *const _, COLOR_TEXT);

    sprintf(buf.as_mut_ptr() as *mut _, "Height : %d px\0".as_ptr() as *const _, h);
    text(8, 60, buf.as_ptr() as *const _, COLOR_TEXT);

    sprintf(buf.as_mut_ptr() as *mut _, "Center : %d,%d\0".as_ptr() as *const _, w / 2, h / 2);
    text(8, 80, buf.as_ptr() as *const _, COLOR_TEXT);

    text(8, 100, "Color  : RGB565\0".as_ptr() as *const _, COLOR_TEXT);
    text(8, 120, "Layer  : Primary\0".as_ptr() as *const _, COLOR_TEXT);

    footer("\0".as_ptr() as *const _, "Back\0".as_ptr() as *const _);
    vm_graphic_flush_layer(&mut LAYER as *mut _, 1);
}

unsafe fn draw_page() {
    match CURRENT_PAGE {
        Page::Menu => draw_menu(),
        Page::Ram => draw_ram(),
        Page::Screen => draw_screen(),
    }
}

extern "C" fn handle_keyevt(event: VMINT, key: VMINT) {
    if event != VM_KEY_EVENT_DOWN as VMINT {
        return;
    }

    unsafe {
        match CURRENT_PAGE {
            Page::Menu => {
                match key as i32 {
                    VM_KEY_UP => {
                        if CURSOR > 0 {
                            CURSOR -= 1;
                        }
                    }
                    VM_KEY_DOWN => {
                        if CURSOR < 1 {
                            CURSOR += 1;
                        }
                    }
                    VM_KEY_OK => {
                        if CURSOR == 0 {
                            CURRENT_PAGE = Page::Ram;
                        } else {
                            CURRENT_PAGE = Page::Screen;
                        }
                    }
                    VM_KEY_RIGHT_SOFTKEY => {
                        vm_exit_app();
                        return;
                    }
                    _ => {}
                }
            }
            Page::Ram | Page::Screen => {
                if key as i32 == VM_KEY_LEFT_SOFTKEY ||
                   key as i32 == VM_KEY_RIGHT_SOFTKEY ||
                   key as i32 == VM_KEY_BACK {
                    CURRENT_PAGE = Page::Menu;
                }
            }
        }
        draw_page();
    }
}

extern "C" fn handle_penevt(_e: VMINT, _x: VMINT, _y: VMINT) {}

extern "C" fn handle_sysevt(msg: VMINT, _param: VMINT) {
    unsafe {
        match msg as u32 {
            VM_MSG_PAINT => {
                if LAYER == -1 {
                    LAYER = vm_graphic_create_layer(
                        0,
                        0,
                        vm_graphic_get_screen_width(),
                        vm_graphic_get_screen_height(),
                        -1,
                    );
                    vm_graphic_set_clip(
                        0,
                        0,
                        vm_graphic_get_screen_width(),
                        vm_graphic_get_screen_height(),
                    );
                }
                draw_page();
            }
            VM_MSG_HIDE | VM_MSG_QUIT => {
                if LAYER != -1 {
                    vm_graphic_delete_layer(LAYER);
                    LAYER = -1;
                }
            }
            _ => {}
        }
    }
}

#[export_name = "vm_main"]
pub extern "C" fn my_vm_main() {
    unsafe {
        vm_reg_sysevt_callback(Some(handle_sysevt));
        vm_reg_keyboard_callback(Some(handle_keyevt));
        vm_reg_pen_callback(Some(handle_penevt));
    }
}

#[no_mangle]
pub static mut return_address: *mut core::ffi::c_void = core::ptr::null_mut();

#[no_mangle]
pub static mut return_sp: *mut u32 = core::ptr::null_mut();

#[no_mangle]
pub extern "C" fn LOGLOG(_file: *const core::ffi::c_char, _line: core::ffi::c_int, _data: *const core::ffi::c_char) {}
