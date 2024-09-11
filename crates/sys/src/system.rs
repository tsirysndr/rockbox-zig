pub fn sleep(ticks: f32) {
    unsafe {
        crate::sleep(ticks);
    }
}

pub fn r#yield() {
    unsafe {
        crate::r#yield();
    }
}

pub fn current_tick() {
    unsafe {
        crate::current_tick();
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
