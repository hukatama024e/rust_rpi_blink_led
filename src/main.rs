extern crate libc;

use std::io;
use std::thread;
use std::time::Duration;
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use std::ptr::{ self, read_volatile, write_volatile };

const GPIO_ADDR : libc::off_t      = 0x3F200000; // BCM2836 and BCM2837 GPIO address
const MEM_BLK_SIZE : libc::size_t  = 4096;       // page size(4KB)

const GPFSEL2_OFFSET_ADDR : isize  = 0x02;       // 0x08 / 4
const GPSET0_OFFSET_ADDR : isize   = 0x07;       // 0x1C / 4
const GPCLR0_OFFSET_ADDR : isize   = 0x0A;       // 0x28 / 4 
const GPLEV0_OFFSET_ADDR : isize   = 0x0D;       // 0x34 / 4

const FSEL27_BIT : isize    = 21;
const SET27_BIT : isize     = 27;
const CLR27_BIT : isize     = 27;
const LEV27_BIT : isize     = 27;

const GPFSEL_INPUT : u32    = 0;
const GPFSEL_OUTPUT : u32   = 1;
const FSEL27_MASK : u32     = 0xFF1FFFFF;

const GPLEV_HIGH : u32      = 1;
const ON : u32              = 1;

const SLEEP_DELAY : u64     = 1;

fn main() {
    let mut blink_cnt : i32 = 0;    
    let gpio_ptr : *mut u32 = map_gpio().expect( "failed to gpio mapping" );
    let fsel2_reg : *mut u32 = unsafe { gpio_ptr.offset( GPFSEL2_OFFSET_ADDR ) };
    let clr0_reg : *mut u32 = unsafe { gpio_ptr.offset( GPCLR0_OFFSET_ADDR ) };

    unsafe {
        write_volatile( fsel2_reg, ( *fsel2_reg & FSEL27_MASK ) | ( GPFSEL_OUTPUT << FSEL27_BIT ) );
    }
    
    while blink_cnt < 10 {
        blink_led( gpio_ptr );
        thread::sleep( Duration::from_secs( SLEEP_DELAY ) );
        blink_cnt = blink_cnt + 1;
    }
    
    unsafe {
        write_volatile( clr0_reg, *clr0_reg | ( ( ON << CLR27_BIT ) ) );
        write_volatile( fsel2_reg, ( *fsel2_reg & FSEL27_MASK ) | ( GPFSEL_INPUT << FSEL27_BIT ) );
    }
}

fn map_gpio() -> io::Result<*mut u32>  {
    let mem_file = OpenOptions::new()
                .read( true )
                .write( true )
                .custom_flags( libc::O_SYNC )
                .open( "/dev/gpiomem" )
                .expect( "can't open /dev/gpiomem" );

    unsafe {
        let gpio_ptr = libc::mmap( ptr::null_mut(),
                                   MEM_BLK_SIZE,
                                   libc::PROT_READ | libc::PROT_WRITE,
                                   libc::MAP_SHARED,
                                   mem_file.as_raw_fd(),
                                   GPIO_ADDR );

        if gpio_ptr == libc::MAP_FAILED {
            Err( io::Error::last_os_error() )
        }
        else
        {
            Ok( gpio_ptr as *mut u32 )
        }
    }
}

fn blink_led( gpio_ptr : *mut u32 ) {
    let set0_reg : *mut u32 = unsafe { gpio_ptr.offset( GPSET0_OFFSET_ADDR ) };
    let clr0_reg : *mut u32 = unsafe { gpio_ptr.offset( GPCLR0_OFFSET_ADDR ) };
    let lev0_reg : *mut u32 = unsafe { gpio_ptr.offset( GPLEV0_OFFSET_ADDR ) };
    let level : u32;
 
    unsafe {
        level = ( read_volatile( lev0_reg ) >> LEV27_BIT ) & 0x00000001;
    }
    
    if level == GPLEV_HIGH {
        unsafe {
            write_volatile( clr0_reg, *clr0_reg | ( ON << CLR27_BIT ) );
        }
    }
    else
    {
        unsafe {
            write_volatile( set0_reg, *set0_reg | ( ON << SET27_BIT ) );
        }
    }
}
