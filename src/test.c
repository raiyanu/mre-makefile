#include "vmsys.h"
#include "vmio.h"
#include "vmgraph.h"
#include "vmstdlib.h"
#include "vmchset.h"

#define COLOR_BG    VM_COLOR_BLACK
#define COLOR_TEXT  VM_COLOR_WHITE
#define COLOR_SEL   VM_COLOR_GREEN
#define COLOR_BAR   0xA554

VMINT layer = -1;
VMWCHAR ucs2[128];

typedef enum
{
    PAGE_MENU,
    PAGE_RAM,
    PAGE_SCREEN
} PAGE;

PAGE page = PAGE_MENU;
VMINT cursor = 0;

const char *menu_items[] =
{
    "RAM Test",
    "Screen"
};

void draw_page(void);

static void text(int x,int y,const char *ascii,VMUINT16 color)
{
    vm_graphic_color c;

    vm_ascii_to_ucs2(ucs2,256,(VMSTR)ascii);

    c.vm_color_565=color;
    vm_graphic_setcolor(&c);

    vm_graphic_textout_to_layer(
        layer,
        x,
        y,
        ucs2,
        vm_graphic_get_screen_width());
}

static void clear(void)
{
    vm_graphic_color c;

    c.vm_color_565=COLOR_BG;
    vm_graphic_setcolor(&c);

    vm_graphic_fill_rect_ex(
        layer,
        0,
        0,
        vm_graphic_get_screen_width(),
        vm_graphic_get_screen_height());
}

static void footer(const char *left,const char *right)
{
    int h=vm_graphic_get_character_height()+4;
    int y=vm_graphic_get_screen_height()-h;

    vm_graphic_color c;

    c.vm_color_565=0x1082;
    vm_graphic_setcolor(&c);

    vm_graphic_fill_rect_ex(
        layer,
        0,
        y,
        vm_graphic_get_screen_width(),
        h);

    text(2,y+2,left,VM_COLOR_WHITE);

    vm_ascii_to_ucs2(ucs2,256,(VMSTR)right);

    c.vm_color_565=VM_COLOR_WHITE;
    vm_graphic_setcolor(&c);

    vm_graphic_textout_to_layer(
        layer,
        vm_graphic_get_screen_width()-vm_graphic_get_string_width(ucs2)-2,
        y+2,
        ucs2,
        vm_graphic_get_screen_width());
}

void draw_menu(void)
{
    clear();

    text(8,8,"Diagnostics",COLOR_SEL);

    int row=vm_graphic_get_character_height()+8;

    for(int i=0;i<2;i++)
    {
        char line[32];

        if(i==cursor)
            sprintf(line,"> %s",menu_items[i]);
        else
            sprintf(line,"  %s",menu_items[i]);

        text(
            8,
            40+i*row,
            line,
            i==cursor?COLOR_SEL:COLOR_TEXT);
    }

    footer("Select","Exit");

    vm_graphic_flush_layer(&layer,1);
}

void draw_ram(void)
{
    clear();

    malloc_stat_t *m=vm_get_malloc_stat();

    char buf[64];

    text(8,8,"Heap Information",COLOR_SEL);

    sprintf(buf,"Current : %d",m->current);
    text(8,40,buf,COLOR_TEXT);

    sprintf(buf,"Peak    : %d",m->peak);
    text(8,60,buf,COLOR_TEXT);

    sprintf(buf,"Free    : %d",m->avail_heap_size);
    text(8,80,buf,COLOR_TEXT);

    sprintf(buf,"Mallocs : %d",m->malloc_count);
    text(8,100,buf,COLOR_TEXT);

    sprintf(buf,"Frees   : %d",m->free_count);
    text(8,120,buf,COLOR_TEXT);

    sprintf(buf,"MRE Mem : %u",vm_get_mre_total_mem_size());
    text(8,140,buf,COLOR_TEXT);

    footer("","Back");

    vm_graphic_flush_layer(&layer,1);
}

void draw_screen(void)
{
    clear();

    char buf[64];

    int w=vm_graphic_get_screen_width();
    int h=vm_graphic_get_screen_height();

    text(8,8,"Screen Information",COLOR_SEL);

    sprintf(buf,"Width  : %d px",w);
    text(8,40,buf,COLOR_TEXT);

    sprintf(buf,"Height : %d px",h);
    text(8,60,buf,COLOR_TEXT);

    sprintf(buf,"Center : %d,%d",w/2,h/2);
    text(8,80,buf,COLOR_TEXT);

    text(8,100,"Color  : RGB565",COLOR_TEXT);

    text(8,120,"Layer  : Primary",COLOR_TEXT);

    footer("","Back");

    vm_graphic_flush_layer(&layer,1);
}

void draw_page(void)
{
    switch(page)
    {
        case PAGE_MENU:
            draw_menu();
            break;

        case PAGE_RAM:
            draw_ram();
            break;

        case PAGE_SCREEN:
            draw_screen();
            break;
    }
}

void handle_keyevt(VMINT event,VMINT key)
{
    if(event!=VM_KEY_EVENT_DOWN)
        return;

    switch(page)
    {
        case PAGE_MENU:

            switch(key)
            {
                case VM_KEY_UP:

                    if(cursor>0)
                        cursor--;

                    break;

                case VM_KEY_DOWN:

                    if(cursor<1)
                        cursor++;

                    break;

                case VM_KEY_OK:

                    if(cursor==0)
                        page=PAGE_RAM;
                    else
                        page=PAGE_SCREEN;

                    break;

                case VM_KEY_RIGHT_SOFTKEY:

                    vm_exit_app();
                    return;
            }

            break;

        case PAGE_RAM:
        case PAGE_SCREEN:

            if(key==VM_KEY_LEFT_SOFTKEY ||
               key==VM_KEY_RIGHT_SOFTKEY ||
               key==VM_KEY_BACK)
            {
                page=PAGE_MENU;
            }

            break;
    }

    draw_page();
}

void handle_penevt(VMINT e,VMINT x,VMINT y)
{
}

void handle_sysevt(VMINT msg,VMINT param)
{
    switch(msg)
    {
        case VM_MSG_PAINT:

            if(layer==-1)
            {
                layer=vm_graphic_create_layer(
                    0,
                    0,
                    vm_graphic_get_screen_width(),
                    vm_graphic_get_screen_height(),
                    -1);

                vm_graphic_set_clip(
                    0,
                    0,
                    vm_graphic_get_screen_width(),
                    vm_graphic_get_screen_height());
            }

            draw_page();
            break;

        case VM_MSG_HIDE:

            if(layer!=-1)
            {
                vm_graphic_delete_layer(layer);
                layer=-1;
            }

            break;

        case VM_MSG_QUIT:

            if(layer!=-1)
            {
                vm_graphic_delete_layer(layer);
                layer=-1;
            }

            break;
    }
}

void vm_main(void)
{
    vm_reg_sysevt_callback(handle_sysevt);
    vm_reg_keyboard_callback(handle_keyevt);
    vm_reg_pen_callback(handle_penevt);
}