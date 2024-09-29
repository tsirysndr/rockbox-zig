use crate::types::{system_status::SystemStatus, RockboxVersion};

pub fn get_rockbox_version() -> RockboxVersion {
    let version = unsafe {
        let version = crate::get_version();
        std::ffi::CStr::from_ptr(version)
            .to_str()
            .unwrap()
            .to_string()
    };
    RockboxVersion { version }
}

pub fn get_global_status() -> SystemStatus {
    unsafe { crate::global_status }.into()
}

pub fn sleep(ticks: i32) {
    unsafe {
        crate::sleep(ticks);
    }
}

pub fn r#yield() {
    unsafe {
        crate::r#yield();
    }
}

pub fn current_tick() -> i64 {
    unsafe {
        let tick_ptr = crate::current_tick();
        match tick_ptr.is_null() {
            true => 0,
            false => std::ptr::read_volatile(tick_ptr),
        }
    }
}

pub fn default_event_handler(event: i64) {
    unsafe {
        crate::default_event_handler(event);
    }
}

pub fn create_thread() {
    unsafe {
        crate::create_thread();
    }
}

pub fn thread_self() {
    unsafe {
        crate::thread_self();
    }
}

pub fn thread_exit() {
    unsafe {
        crate::thread_exit();
    }
}

pub fn thread_wait() {
    unsafe {
        crate::thread_wait();
    }
}

pub fn thread_thaw() {
    unsafe {
        crate::thread_thaw();
    }
}

pub fn thread_set_priority(thread_id: u32, priority: i32) {
    unsafe {
        crate::thread_set_priority(thread_id, priority);
    }
}

pub fn mutex_init() {
    unsafe {
        crate::mutex_init();
    }
}

pub fn mutex_lock() {
    unsafe {
        crate::mutex_lock();
    }
}

pub fn mutex_unlock() {
    unsafe {
        crate::mutex_unlock();
    }
}

pub fn semaphore_init() {
    unsafe {
        crate::semaphore_init();
    }
}

pub fn semaphore_wait() {
    unsafe {
        crate::semaphore_wait();
    }
}

pub fn semaphore_release() {
    unsafe {
        crate::semaphore_release();
    }
}

pub fn reset_poweroff_timer() {
    unsafe {
        crate::reset_poweroff_timer();
    }
}

pub fn set_sleeptimer_duration() {
    unsafe {
        crate::set_sleeptimer_duration();
    }
}

pub fn get_sleep_timer() {
    unsafe {
        crate::get_sleep_timer();
    }
}
