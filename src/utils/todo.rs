/*!
 * TODO and debug logging utilities for RXServer
 * 
 * This module provides consistent TODO markers and debug logging
 * for tracking non-working or incomplete functionality.
 */

use tracing::{debug, warn, error, info};

/// Macro for marking TODO items with consistent logging
#[macro_export]
macro_rules! todo_log {
    ($level:ident, $category:expr, $msg:expr) => {
        tracing::$level!(
            todo_category = $category,
            todo_message = $msg,
            file = file!(),
            line = line!(),
            "TODO: {}", $msg
        );
    };
    ($level:ident, $category:expr, $msg:expr, $($arg:tt)*) => {
        tracing::$level!(
            todo_category = $category,
            todo_message = format!($msg, $($arg)*),
            file = file!(),
            line = line!(),
            "TODO: {}", format!($msg, $($arg)*)
        );
    };
}

/// Mark a critical missing implementation
#[macro_export]
macro_rules! todo_critical {
    ($category:expr, $msg:expr) => {
        $crate::todo_log!(error, $category, $msg);
        //panic!("CRITICAL TODO: {}", $msg);
    };
    ($category:expr, $msg:expr, $($arg:tt)*) => {
        $crate::todo_log!(error, $category, $msg, $($arg)*);
        //panic!("CRITICAL TODO: {}", format!($msg, $($arg)*));
    };
}

/// Mark a high priority TODO
#[macro_export]
macro_rules! todo_high {
    ($category:expr, $msg:expr) => {
        $crate::todo_log!(warn, $category, $msg);
    };
    ($category:expr, $msg:expr, $($arg:tt)*) => {
        $crate::todo_log!(warn, $category, $msg, $($arg)*);
    };
}

/// Mark a medium priority TODO
#[macro_export]
macro_rules! todo_medium {
    ($category:expr, $msg:expr) => {
        $crate::todo_log!(info, $category, $msg);
    };
    ($category:expr, $msg:expr, $($arg:tt)*) => {
        $crate::todo_log!(info, $category, $msg, $($arg)*);
    };
}

/// Mark a low priority TODO
#[macro_export]
macro_rules! todo_low {
    ($category:expr, $msg:expr) => {
        $crate::todo_log!(debug, $category, $msg);
    };
    ($category:expr, $msg:expr, $($arg:tt)*) => {
        $crate::todo_log!(debug, $category, $msg, $($arg)*);
    };
}

/// Return a not implemented error with TODO logging
#[macro_export]
macro_rules! todo_return {
    ($category:expr, $msg:expr) => {{
        $crate::todo_high!($category, $msg);
        Err(crate::Error::NotImplemented($msg.to_string()))
    }};
    ($category:expr, $msg:expr, $($arg:tt)*) => {{
        let msg = format!($msg, $($arg)*);
        $crate::todo_high!($category, "{}", msg);
        Err(crate::Error::NotImplemented(msg))
    }};
}

/// Placeholder implementation that logs TODO and returns default
#[macro_export]
macro_rules! todo_placeholder {
    ($category:expr, $msg:expr, $default:expr) => {{
        $crate::todo_medium!($category, $msg);
        $default
    }};
    ($category:expr, $msg:expr, $default:expr, $($arg:tt)*) => {{
        $crate::todo_medium!($category, $msg, $($arg)*);
        $default
    }};
}
