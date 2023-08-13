use std::fmt;

pub fn die<E: fmt::Debug>(error: E) -> ! {
    log::error!("{:?}", error);
    std::process::exit(1);
}