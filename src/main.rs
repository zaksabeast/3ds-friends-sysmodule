#![no_std]
#![allow(incomplete_features)]
#![feature(alloc_error_handler)]
#![feature(start)]
#![feature(if_let_guard)]

extern crate alloc;

mod frd;
mod heap_allocator;
mod log;

use alloc::{boxed::Box, vec};
#[cfg(not(test))]
use core::{arch::asm, panic::PanicInfo};
use ctr::{
    ac, fs,
    http::httpc_init,
    memory::{MemoryBlock, MemoryPermission},
    ptm, srv, svc,
    sysmodule::{
        notification::NotificationManager,
        server::{Service, ServiceManager},
    },
};
use frd::{
    context::FriendServiceContext, frda::handle_frda_request, frdn::handle_frdn_request,
    frdu::handle_frdu_request, notification::handle_sleep_notification,
};

/// Called after main exits to clean things up.
/// Used by 3ds toolchain.
#[no_mangle]
pub extern "C" fn __wrap_exit() {
    svc::exit_process();
}

#[repr(align(0x1000))]
struct HttpBuffer([u8; 0x1000]);

impl HttpBuffer {
    fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

static mut HTTP_BUFFER: HttpBuffer = HttpBuffer([0; 0x1000]);

/// Called before main to initialize the system.
/// Used by 3ds toolchain.
#[no_mangle]
pub extern "C" fn initSystem() {
    // This is safe because we're only supposed to use this one time
    // while initializing the system, which is happening right here.
    unsafe { heap_allocator::init_heap() };

    loop {
        match srv::init() {
            Ok(_) => break,
            Err(error_code) => {
                if error_code != 0xd88007fa {
                    panic!();
                }
            }
        };

        svc::sleep_thread(500000i64);
    }

    fs::init().unwrap();
    ac::init().unwrap();

    // This is safe as long as we're single threaded
    let aligned_buffer = unsafe { HTTP_BUFFER.as_mut_slice() };
    let memory_block = MemoryBlock::new(
        aligned_buffer,
        MemoryPermission::None,
        MemoryPermission::ReadWrite,
    )
    .expect("");
    httpc_init(memory_block).expect("HTTPC did not init");
}

#[cfg(not(test))]
#[panic_handler]
fn panic(panic: &PanicInfo<'_>) -> ! {
    if let Some(location) = panic.location() {
        let file = location.file();
        let slice = &file[file.len() - 7..];

        // Since we're about to break, storing a few u32s in these registers won't break us further.
        // In the future it might be helpful to disable this for release builds.
        unsafe {
            // r9 and r10 aren't used as frequently as the lower registers, so in most situations
            // we'll get more useful information by storing the last 4 characters of the file name
            // and the line number where we broke.
            let partial_file_name = *(slice.as_ptr() as *const u32);
            asm!("mov r9, {}", in(reg) partial_file_name);
            asm!("mov r10, {}", in(reg) location.line());
        }
    }

    svc::break_execution(svc::UserBreakType::Panic)
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn abort() -> ! {
    svc::break_execution(svc::UserBreakType::Panic)
}

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    log::debug("\n\nStarted!");

    let global_context = Box::new(FriendServiceContext::new().unwrap());

    let services = vec![
        Service::new("frd:u", 8, handle_frdu_request).unwrap(),
        Service::new("frd:a", 8, handle_frda_request).unwrap(),
        Service::new("frd:n", 1, handle_frdn_request).unwrap(),
    ];

    log::debug("Setting up notification manager");

    let mut notification_manger = NotificationManager::new().unwrap();

    notification_manger
        .subscribe(
            ptm::NotificationId::SleepRequested,
            handle_sleep_notification,
        )
        .unwrap();
    notification_manger
        .subscribe(ptm::NotificationId::GoingToSleep, handle_sleep_notification)
        .unwrap();
    notification_manger
        .subscribe(
            ptm::NotificationId::FullyWakingUp,
            handle_sleep_notification,
        )
        .unwrap();
    // TODO:
    // notification_manger.subscribe(0x301, do_something);
    // notification_manger.subscribe(0x302, do_something);

    log::debug("Setting up service manager");
    let mut manager = ServiceManager::new(services, notification_manger, global_context);
    log::debug("Set up service manager");
    let result = manager.run();

    match result {
        Ok(_) => 0,
        Err(_) => panic!(),
    }
}
