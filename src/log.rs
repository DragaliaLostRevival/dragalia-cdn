pub extern crate colored;

#[macro_use]
mod log {
    macro_rules! error {
        ($message:expr, $($args:tt)*) => {{
            eprintln!(
                    "[{}] {} {}",
                $crate::timestamp::now(),
                "ERROR".red(),
                format_args!($message, $($args)*)
            );
        }};
        ($message:expr) => {{
            println!("[{}] {} {}", $crate::timestamp::now(), "ERROR".red(), $message);
        }}
    }

    macro_rules! warn {
        ($message:expr, $($args:tt)*) => {{
            println!(
                "[{}] {} {}",
                $crate::timestamp::now(),
                "WARN".yellow(),
                format_args!($message, $($args)*)
            );
        }};
        ($message:expr) => {{
            println!("[{}] {} {}", $crate::timestamp::now(), "WARN".yellow(), $message);
        }}
    }

    macro_rules! info {
        ($message:expr, $($args:tt)*) => {{
            println!(
                "[{}] {} {}",
                $crate::timestamp::now(),
                "INFO",
                format_args!($message, $($args)*)
            );
        }};
        ($message:expr) => {{
            println!("[{}] {} {}", $crate::timestamp::now(), "INFO", $message);
        }}
    }
}

