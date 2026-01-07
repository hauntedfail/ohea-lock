//! # Ohea Lock
//!
//! A Rust library for controlling Ohea Lock BLE smart locks.
//!
//! This library provides a protocol-focused implementation for communicating with
//! Ohea Lock devices over Bluetooth Low Energy. The core library is transport-agnostic,
//! with optional btleplug support for desktop platforms.
//!
//! ## Features
//!
//! - `btleplug-support`: Enable btleplug-based transport for desktop platforms.
//!
//! ## Architecture
//!
//! The library is structured in layers:
//!
//! 1. **Protocol Layer** ([`protocol`]): UUIDs, constants, and protocol types.
//! 2. **Transport Layer** ([`transport`]): Abstract trait for BLE operations.
//! 3. **High-Level API** ([`OheaLock`]): Convenient interface for lock operations.
//!
//! ## Example
//!
//! ```ignore
//! use ohea_lock::{OheaLock, btleplug::BtleplugTransport};
//!
//! // Assuming `peripheral` is an already-paired btleplug Peripheral
//! let transport = BtleplugTransport::connect_and_discover(peripheral).await?;
//! let lock = OheaLock::new(transport);
//!
//! // Get device info
//! let info = lock.get_device_info().await?;
//! println!("Battery: {}%", info.battery_level);
//!
//! // Control the lock
//! lock.unlock().await?;
//! lock.lock().await?;
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod error;
pub mod protocol;
pub mod transport;

#[cfg(feature = "btleplug-support")]
pub mod btleplug_impl;

// Re-exports for convenience
pub use error::{Error, Result};
pub use protocol::{DeviceInfo, LockState};
pub use transport::{Transport, TransportExt};

#[cfg(feature = "btleplug-support")]
pub use btleplug_impl::BtleplugTransport;

use crate::protocol::{
    BATTERY_LEVEL_CHAR_UUID, COMMAND_CHAR_UUID, DEVICE_NAME_CHAR_UUID, FIRMWARE_REVISION_CHAR_UUID,
    LOCK_POSITION_CHAR_UUID, LOCK_STATE_CHAR_UUID, STATUS_CHAR_UUID,
};

/// High-level interface for controlling an Ohea Lock device.
///
/// This struct wraps a [`Transport`] implementation and provides convenient
/// methods for common lock operations.
///
/// # Example
///
/// ```ignore
/// let lock = OheaLock::new(transport);
/// let state = lock.get_lock_state().await?;
/// if state.is_locked() {
///     lock.unlock().await?;
/// }
/// ```
pub struct OheaLock<T: Transport> {
    transport: T,
    initialized: bool,
}

impl<T: Transport> OheaLock<T> {
    /// Create a new `OheaLock` instance with the given transport.
    #[must_use]
    pub const fn new(transport: T) -> Self {
        Self {
            transport,
            initialized: false,
        }
    }

    /// Get the underlying transport.
    #[must_use]
    pub const fn transport(&self) -> &T {
        &self.transport
    }

    /// Get a mutable reference to the underlying transport.
    pub fn transport_mut(&mut self) -> &mut T {
        &mut self.transport
    }

    /// Initialize the session with the lock.
    ///
    /// This sends the initialization command to the command register.
    /// Should be called after connecting to a newly paired device.
    ///
    /// # Errors
    ///
    /// Returns an error if the write fails.
    pub async fn initialize(&mut self) -> Result<()> {
        let cmd = protocol::build_init_command();
        self.transport.write(COMMAND_CHAR_UUID, &cmd).await?;
        self.initialized = true;
        Ok(())
    }

    /// Get the current lock state.
    ///
    /// # Errors
    ///
    /// Returns an error if the read fails or returns invalid data.
    pub async fn get_lock_state(&self) -> Result<LockState> {
        let data = self.transport.read(LOCK_STATE_CHAR_UUID).await?;
        let byte = data
            .first()
            .copied()
            .ok_or_else(|| Error::InvalidResponse("empty lock state".to_string()))?;
        LockState::try_from(byte)
    }

    /// Get the current lock position.
    ///
    /// This may differ from lock state during transitions.
    ///
    /// # Errors
    ///
    /// Returns an error if the read fails.
    pub async fn get_lock_position(&self) -> Result<u8> {
        self.transport.read_byte(LOCK_POSITION_CHAR_UUID).await
    }

    /// Unlock the lock.
    ///
    /// # Errors
    ///
    /// Returns an error if the write fails.
    pub async fn unlock(&self) -> Result<()> {
        self.transport
            .write(LOCK_STATE_CHAR_UUID, &[LockState::Unlocked.as_byte()])
            .await
    }

    /// Lock the lock.
    ///
    /// # Errors
    ///
    /// Returns an error if the write fails.
    pub async fn lock(&self) -> Result<()> {
        self.transport
            .write(LOCK_STATE_CHAR_UUID, &[LockState::Locked.as_byte()])
            .await
    }

    /// Set the lock state.
    ///
    /// # Errors
    ///
    /// Returns an error if the write fails.
    pub async fn set_lock_state(&self, state: LockState) -> Result<()> {
        self.transport
            .write(LOCK_STATE_CHAR_UUID, &[state.as_byte()])
            .await
    }

    /// Get the battery level as a percentage (0-100).
    ///
    /// # Errors
    ///
    /// Returns an error if the read fails.
    pub async fn get_battery_level(&self) -> Result<u8> {
        self.transport.read_byte(BATTERY_LEVEL_CHAR_UUID).await
    }

    /// Get the firmware version string.
    ///
    /// # Errors
    ///
    /// Returns an error if the read fails or the response is not valid UTF-8.
    pub async fn get_firmware_version(&self) -> Result<String> {
        self.transport.read_string(FIRMWARE_REVISION_CHAR_UUID).await
    }

    /// Get the device name.
    ///
    /// # Errors
    ///
    /// Returns an error if the read fails or the response is not valid UTF-8.
    pub async fn get_device_name(&self) -> Result<String> {
        self.transport.read_string(DEVICE_NAME_CHAR_UUID).await
    }

    /// Get comprehensive device information.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the underlying reads fail.
    pub async fn get_device_info(&self) -> Result<DeviceInfo> {
        let name = self.get_device_name().await?;
        let firmware_version = self.get_firmware_version().await?;
        let battery_level = self.get_battery_level().await?;

        Ok(DeviceInfo {
            name,
            firmware_version,
            battery_level,
        })
    }

    /// Get the current status byte.
    ///
    /// # Errors
    ///
    /// Returns an error if the read fails.
    pub async fn get_status(&self) -> Result<u8> {
        self.transport.read_byte(STATUS_CHAR_UUID).await
    }

    /// Subscribe to lock state notifications.
    ///
    /// # Errors
    ///
    /// Returns an error if subscription fails.
    pub async fn subscribe_lock_state(&self) -> Result<()> {
        self.transport.subscribe(LOCK_STATE_CHAR_UUID).await
    }

    /// Subscribe to status notifications.
    ///
    /// # Errors
    ///
    /// Returns an error if subscription fails.
    pub async fn subscribe_status(&self) -> Result<()> {
        self.transport.subscribe(STATUS_CHAR_UUID).await
    }

    /// Subscribe to battery level notifications.
    ///
    /// # Errors
    ///
    /// Returns an error if subscription fails.
    pub async fn subscribe_battery_level(&self) -> Result<()> {
        self.transport.subscribe(BATTERY_LEVEL_CHAR_UUID).await
    }

    /// Unsubscribe from all notifications.
    ///
    /// # Errors
    ///
    /// Returns an error if any unsubscription fails.
    pub async fn unsubscribe_all(&self) -> Result<()> {
        // Ignore errors for individual unsubscriptions
        let _ = self.transport.unsubscribe(LOCK_STATE_CHAR_UUID).await;
        let _ = self.transport.unsubscribe(STATUS_CHAR_UUID).await;
        let _ = self.transport.unsubscribe(BATTERY_LEVEL_CHAR_UUID).await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    /// Mock transport for testing OheaLock API.
    struct MockTransport {
        responses: Arc<RwLock<HashMap<uuid::Uuid, Vec<u8>>>>,
        writes: Arc<RwLock<Vec<(uuid::Uuid, Vec<u8>)>>>,
        subscribed: Arc<RwLock<Vec<uuid::Uuid>>>,
    }

    impl MockTransport {
        fn new() -> Self {
            Self {
                responses: Arc::new(RwLock::new(HashMap::new())),
                writes: Arc::new(RwLock::new(Vec::new())),
                subscribed: Arc::new(RwLock::new(Vec::new())),
            }
        }

        async fn set_response(&self, uuid: uuid::Uuid, data: Vec<u8>) {
            self.responses.write().await.insert(uuid, data);
        }

        async fn last_write(&self) -> Option<(uuid::Uuid, Vec<u8>)> {
            self.writes.read().await.last().cloned()
        }
    }

    #[async_trait]
    impl Transport for MockTransport {
        async fn read(&self, char_uuid: uuid::Uuid) -> Result<Vec<u8>> {
            self.responses
                .read()
                .await
                .get(&char_uuid)
                .cloned()
                .ok_or_else(|| Error::CharacteristicNotFound("mock"))
        }

        async fn write(&self, char_uuid: uuid::Uuid, data: &[u8]) -> Result<()> {
            self.writes.write().await.push((char_uuid, data.to_vec()));
            Ok(())
        }

        async fn write_without_response(&self, char_uuid: uuid::Uuid, data: &[u8]) -> Result<()> {
            self.write(char_uuid, data).await
        }

        async fn subscribe(&self, char_uuid: uuid::Uuid) -> Result<()> {
            self.subscribed.write().await.push(char_uuid);
            Ok(())
        }

        async fn unsubscribe(&self, char_uuid: uuid::Uuid) -> Result<()> {
            self.subscribed.write().await.retain(|&u| u != char_uuid);
            Ok(())
        }

        fn is_connected(&self) -> bool {
            true
        }
    }

    // =========================================================================
    // LockState Unit Tests
    // =========================================================================

    #[test]
    fn lock_state_conversion() {
        assert_eq!(LockState::from_byte(0x00), Some(LockState::Locked));
        assert_eq!(LockState::from_byte(0x01), Some(LockState::Unlocked));
        assert_eq!(LockState::from_byte(0x02), None);

        assert_eq!(LockState::Locked.as_byte(), 0x00);
        assert_eq!(LockState::Unlocked.as_byte(), 0x01);
    }

    #[test]
    fn lock_state_predicates() {
        assert!(LockState::Locked.is_locked());
        assert!(!LockState::Locked.is_unlocked());
        assert!(!LockState::Unlocked.is_locked());
        assert!(LockState::Unlocked.is_unlocked());
    }

    // =========================================================================
    // OheaLock Constructor Tests
    // =========================================================================

    #[test]
    fn ohea_lock_new_is_not_initialized() {
        let transport = MockTransport::new();
        let lock = OheaLock::new(transport);
        assert!(!lock.initialized);
    }

    #[test]
    fn ohea_lock_transport_accessor() {
        let transport = MockTransport::new();
        let lock = OheaLock::new(transport);
        assert!(lock.transport().is_connected());
    }

    // =========================================================================
    // OheaLock Async Operation Tests
    // =========================================================================

    #[tokio::test]
    async fn initialize_sends_correct_command() {
        let transport = MockTransport::new();
        let mut lock = OheaLock::new(transport);

        lock.initialize().await.unwrap();

        assert!(lock.initialized);
        let (uuid, data) = lock.transport().last_write().await.unwrap();
        assert_eq!(uuid, COMMAND_CHAR_UUID);
        assert_eq!(data, protocol::build_init_command());
    }

    #[tokio::test]
    async fn get_lock_state_returns_locked() {
        let transport = MockTransport::new();
        transport.set_response(LOCK_STATE_CHAR_UUID, vec![0x00]).await;
        let lock = OheaLock::new(transport);

        let state = lock.get_lock_state().await.unwrap();
        assert_eq!(state, LockState::Locked);
    }

    #[tokio::test]
    async fn get_lock_state_returns_unlocked() {
        let transport = MockTransport::new();
        transport.set_response(LOCK_STATE_CHAR_UUID, vec![0x01]).await;
        let lock = OheaLock::new(transport);

        let state = lock.get_lock_state().await.unwrap();
        assert_eq!(state, LockState::Unlocked);
    }

    #[tokio::test]
    async fn get_lock_state_errors_on_invalid_value() {
        let transport = MockTransport::new();
        transport.set_response(LOCK_STATE_CHAR_UUID, vec![0xFF]).await;
        let lock = OheaLock::new(transport);

        let result = lock.get_lock_state().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn unlock_writes_correct_value() {
        let transport = MockTransport::new();
        let lock = OheaLock::new(transport);

        lock.unlock().await.unwrap();

        let (uuid, data) = lock.transport().last_write().await.unwrap();
        assert_eq!(uuid, LOCK_STATE_CHAR_UUID);
        assert_eq!(data, vec![0x01]);
    }

    #[tokio::test]
    async fn lock_writes_correct_value() {
        let transport = MockTransport::new();
        let lock = OheaLock::new(transport);

        lock.lock().await.unwrap();

        let (uuid, data) = lock.transport().last_write().await.unwrap();
        assert_eq!(uuid, LOCK_STATE_CHAR_UUID);
        assert_eq!(data, vec![0x00]);
    }

    #[tokio::test]
    async fn get_battery_level_returns_percentage() {
        let transport = MockTransport::new();
        transport.set_response(BATTERY_LEVEL_CHAR_UUID, vec![0x64]).await; // 100%
        let lock = OheaLock::new(transport);

        let level = lock.get_battery_level().await.unwrap();
        assert_eq!(level, 100);
    }

    #[tokio::test]
    async fn get_firmware_version_returns_string() {
        let transport = MockTransport::new();
        transport.set_response(FIRMWARE_REVISION_CHAR_UUID, b"1.0".to_vec()).await;
        let lock = OheaLock::new(transport);

        let version = lock.get_firmware_version().await.unwrap();
        assert_eq!(version, "1.0");
    }

    #[tokio::test]
    async fn get_device_name_returns_ohea_lock() {
        let transport = MockTransport::new();
        transport.set_response(DEVICE_NAME_CHAR_UUID, b"Ohea Lock".to_vec()).await;
        let lock = OheaLock::new(transport);

        let name = lock.get_device_name().await.unwrap();
        assert_eq!(name, "Ohea Lock");
    }

    #[tokio::test]
    async fn get_device_info_aggregates_all_fields() {
        let transport = MockTransport::new();
        transport.set_response(DEVICE_NAME_CHAR_UUID, b"Ohea Lock".to_vec()).await;
        transport.set_response(FIRMWARE_REVISION_CHAR_UUID, b"1.0".to_vec()).await;
        transport.set_response(BATTERY_LEVEL_CHAR_UUID, vec![0x64]).await;
        let lock = OheaLock::new(transport);

        let info = lock.get_device_info().await.unwrap();

        assert_eq!(info.name, "Ohea Lock");
        assert_eq!(info.firmware_version, "1.0");
        assert_eq!(info.battery_level, 100);
    }

    // =========================================================================
    // Subscription Tests
    // =========================================================================

    #[tokio::test]
    async fn subscribe_lock_state_subscribes_to_correct_uuid() {
        let transport = MockTransport::new();
        let lock = OheaLock::new(transport);

        lock.subscribe_lock_state().await.unwrap();

        let subscribed = lock.transport().subscribed.read().await;
        assert!(subscribed.contains(&LOCK_STATE_CHAR_UUID));
    }

    #[tokio::test]
    async fn unsubscribe_all_does_not_error() {
        let transport = MockTransport::new();
        let lock = OheaLock::new(transport);

        // Should not error even if nothing is subscribed
        let result = lock.unsubscribe_all().await;
        assert!(result.is_ok());
    }
}

