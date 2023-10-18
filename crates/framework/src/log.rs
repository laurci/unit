use crate::vm_internals;

pub fn write_log(message: &str) {
    unsafe {
        vm_internals::unit_log(message.as_ptr() as _, message.len() as _);
    }
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => ({
        unit::log::write_log(&format!($($arg)*));
    })
}
