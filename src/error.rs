//! Error types for the Ohea Lock library.

use thiserror::Error;

/// Result type alias for Ohea Lock operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when interacting with an Ohea Lock device.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// The requested characteristic was not found on the device.
    #[error("characteristic not found: {0}")]
    CharacteristicNotFound(&'static str),

    /// The requested service was not found on the device.
    #[error("service not found: {0}")]
    ServiceNotFound(&'static str),

    /// Authentication is required but the device is not paired.
    #[error("insufficient authentication - pairing required")]
    InsufficientAuthentication,

    /// The device is not connected.
    #[error("device not connected")]
    NotConnected,

    /// A BLE transport error occurred.
    #[error("transport error: {0}")]
    Transport(String),

    /// Invalid response received from the device.
    #[error("invalid response: {0}")]
    InvalidResponse(String),

    /// The operation timed out.
    #[error("operation timed out")]
    Timeout,

    /// An I/O error occurred.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// btleplug-specific error (only available with `btleplug-support` feature).
    #[cfg(feature = "btleplug-support")]
    #[error("btleplug error: {0}")]
    Btleplug(#[from] btleplug::Error),
}

impl Error {
    /// Returns `true` if this error indicates that pairing is required.
    #[must_use]
    pub const fn requires_pairing(&self) -> bool {
        matches!(self, Self::InsufficientAuthentication)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_messages() {
        let cases = [
            (
                Error::CharacteristicNotFound("lock_state"),
                "characteristic not found: lock_state",
            ),
            (
                Error::ServiceNotFound("battery"),
                "service not found: battery",
            ),
            (
                Error::InsufficientAuthentication,
                "insufficient authentication - pairing required",
            ),
            (Error::NotConnected, "device not connected"),
            (
                Error::Transport("timeout".into()),
                "transport error: timeout",
            ),
            (
                Error::InvalidResponse("empty".into()),
                "invalid response: empty",
            ),
            (Error::Timeout, "operation timed out"),
        ];
        for (error, expected) in cases {
            assert_eq!(error.to_string(), expected);
        }
    }

    #[test]
    fn requires_pairing_returns_true_for_auth_error() {
        assert!(Error::InsufficientAuthentication.requires_pairing());
    }

    #[test]
    fn requires_pairing_returns_false_for_other_errors() {
        let non_auth_errors = [
            Error::CharacteristicNotFound("test"),
            Error::ServiceNotFound("test"),
            Error::NotConnected,
            Error::Transport("test".into()),
            Error::InvalidResponse("test".into()),
            Error::Timeout,
        ];
        for error in non_auth_errors {
            assert!(!error.requires_pairing(), "unexpected true for {error:?}");
        }
    }

    #[test]
    fn io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Io(_)));
    }

    #[test]
    fn error_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        // This will fail to compile if Error is not Send + Sync
        assert_send_sync::<Error>();
    }
}
