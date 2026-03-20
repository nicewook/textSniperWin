/// 디버그 빌드에서만 출력되는 로그 매크로
#[cfg(debug_assertions)]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        eprintln!($($arg)*)
    };
}

#[cfg(not(debug_assertions))]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        let _ = format_args!($($arg)*);
    };
}

pub(crate) use debug_log;

#[cfg(test)]
mod tests {
    use super::debug_log;

    #[test]
    fn test_debug_log_compiles_with_string() {
        debug_log!("test message");
    }

    #[test]
    fn test_debug_log_compiles_with_format_args() {
        debug_log!("value: {}", 42);
        debug_log!("multi: {} {} {:?}", "a", 1, vec![1, 2]);
    }
}
