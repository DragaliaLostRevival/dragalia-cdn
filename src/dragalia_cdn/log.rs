macro_rules! error {
        ($message:expr, $($args:tt)*) => {{
            eprintln!(
                    "[{}] {} {}",
                $crate::dragalia_cdn::timestamp::now(),
                "ERROR".red(),
                format_args!($message, $($args)*)
            );
        }};
        ($message:expr) => {{
            println!("[{}] {} {}", $crate::dragalia_cdn::timestamp::now(), "ERROR".red(), $message);
        }}
}

macro_rules! warn {
        ($message:expr, $($args:tt)*) => {{
            println!(
                "[{}] {} {}",
                $crate::dragalia_cdn::timestamp::now(),
                "WARN".yellow(),
                format_args!($message, $($args)*)
            );
        }};
        ($message:expr) => {{
            println!("[{}] {} {}", $crate::dragalia_cdn::timestamp::now(), "WARN".yellow(), $message);
        }}
}

macro_rules! info {
        ($message:expr, $($args:tt)*) => {{
            println!(
                "[{}] {} {}",
                $crate::dragalia_cdn::timestamp::now(),
                "INFO",
                format_args!($message, $($args)*)
            );
        }};
        ($message:expr) => {{
            println!("[{}] {} {}", $crate::dragalia_cdn::timestamp::now(), "INFO", $message);
        }}
}
