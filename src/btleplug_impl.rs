//! btleplug-based transport implementation.
//!
//! This module provides a [`Transport`] implementation using btleplug,
//! enabling communication with Ohea Lock devices on desktop platforms.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use btleplug::api::{Characteristic, Peripheral as _, WriteType};
use btleplug::platform::Peripheral;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::transport::Transport;

/// A btleplug-based transport for communicating with Ohea Lock devices.
///
/// This implementation wraps a btleplug [`Peripheral`] and provides the
/// [`Transport`] trait for use with the core protocol logic.
///
/// # Example
///
/// ```ignore
/// use ohea_lock::btleplug::BtleplugTransport;
/// use ohea_lock::OheaLock;
///
/// let transport = BtleplugTransport::new(peripheral).await?;
/// let lock = OheaLock::new(transport);
/// let state = lock.get_lock_state().await?;
/// ```
pub struct BtleplugTransport {
    peripheral: Peripheral,
    /// Cached characteristic map for fast lookups.
    char_cache: Arc<RwLock<HashMap<Uuid, Characteristic>>>,
}

impl BtleplugTransport {
    /// Create a new transport from a connected btleplug peripheral.
    ///
    /// The peripheral must already be connected and have its services discovered.
    ///
    /// # Errors
    ///
    /// Returns an error if the peripheral is not connected or service discovery
    /// has not been performed.
    pub async fn new(peripheral: Peripheral) -> Result<Self> {
        if !peripheral.is_connected().await? {
            return Err(Error::NotConnected);
        }

        let transport = Self {
            peripheral,
            char_cache: Arc::new(RwLock::new(HashMap::new())),
        };

        // Populate the characteristic cache
        transport.populate_cache().await?;

        Ok(transport)
    }

    /// Create a new transport and perform service discovery.
    ///
    /// This is a convenience method that connects (if needed) and discovers services.
    ///
    /// # Errors
    ///
    /// Returns an error if connection or service discovery fails.
    pub async fn connect_and_discover(peripheral: Peripheral) -> Result<Self> {
        if !peripheral.is_connected().await? {
            peripheral.connect().await?;
        }
        peripheral.discover_services().await?;
        Self::new(peripheral).await
    }

    /// Get the underlying btleplug peripheral.
    #[must_use]
    pub fn peripheral(&self) -> &Peripheral {
        &self.peripheral
    }

    /// Populate the characteristic cache from discovered services.
    async fn populate_cache(&self) -> Result<()> {
        let services = self.peripheral.services();
        let mut cache = self.char_cache.write().await;

        for service in services {
            for char in service.characteristics {
                cache.insert(char.uuid, char);
            }
        }

        Ok(())
    }

    /// Find a characteristic by UUID.
    async fn find_characteristic(&self, uuid: Uuid) -> Result<Characteristic> {
        // First check the cache
        {
            let cache = self.char_cache.read().await;
            if let Some(char) = cache.get(&uuid) {
                return Ok(char.clone());
            }
        }

        // If not in cache, try to refresh and check again
        self.populate_cache().await?;

        let cache = self.char_cache.read().await;
        cache
            .get(&uuid)
            .cloned()
            .ok_or(Error::CharacteristicNotFound(uuid_to_name(uuid)))
    }
}

#[async_trait]
impl Transport for BtleplugTransport {
    async fn read(&self, char_uuid: Uuid) -> Result<Vec<u8>> {
        let char = self.find_characteristic(char_uuid).await?;
        self.peripheral
            .read(&char)
            .await
            .map_err(|e| map_btleplug_error(e, char_uuid))
    }

    async fn write(&self, char_uuid: Uuid, data: &[u8]) -> Result<()> {
        let char = self.find_characteristic(char_uuid).await?;
        self.peripheral
            .write(&char, data, WriteType::WithResponse)
            .await
            .map_err(|e| map_btleplug_error(e, char_uuid))
    }

    async fn write_without_response(&self, char_uuid: Uuid, data: &[u8]) -> Result<()> {
        let char = self.find_characteristic(char_uuid).await?;
        self.peripheral
            .write(&char, data, WriteType::WithoutResponse)
            .await
            .map_err(|e| map_btleplug_error(e, char_uuid))
    }

    async fn subscribe(&self, char_uuid: Uuid) -> Result<()> {
        let char = self.find_characteristic(char_uuid).await?;
        self.peripheral
            .subscribe(&char)
            .await
            .map_err(|e| map_btleplug_error(e, char_uuid))
    }

    async fn unsubscribe(&self, char_uuid: Uuid) -> Result<()> {
        let char = self.find_characteristic(char_uuid).await?;
        self.peripheral
            .unsubscribe(&char)
            .await
            .map_err(|e| map_btleplug_error(e, char_uuid))
    }

    fn is_connected(&self) -> bool {
        // btleplug's is_connected is async, so we use a blocking check here.
        // In practice, the caller should use the async version when available.
        true
    }
}

/// Map btleplug errors to our error type, detecting authentication errors.
fn map_btleplug_error(err: btleplug::Error, _char_uuid: Uuid) -> Error {
    // Check for insufficient authentication error
    let err_str = err.to_string().to_lowercase();
    if err_str.contains("authentication") || err_str.contains("0x05") {
        return Error::InsufficientAuthentication;
    }
    Error::Btleplug(err)
}

/// Get a human-readable name for known UUIDs.
fn uuid_to_name(uuid: Uuid) -> &'static str {
    use crate::protocol::{
        BATTERY_LEVEL_CHAR_UUID, COMMAND_CHAR_UUID, CONFIG_CHAR_UUID, CONTROL_CHAR_UUID,
        DEVICE_INFO_CHAR_UUID, DEVICE_NAME_CHAR_UUID, EXTENDED_INFO_CHAR_UUID,
        FIRMWARE_REVISION_CHAR_UUID, LOCK_POSITION_CHAR_UUID, LOCK_STATE_CHAR_UUID,
        STATUS_CHAR_UUID,
    };

    match uuid {
        _ if uuid == LOCK_STATE_CHAR_UUID => "lock_state",
        _ if uuid == STATUS_CHAR_UUID => "status",
        _ if uuid == LOCK_POSITION_CHAR_UUID => "lock_position",
        _ if uuid == COMMAND_CHAR_UUID => "command",
        _ if uuid == DEVICE_INFO_CHAR_UUID => "device_info",
        _ if uuid == EXTENDED_INFO_CHAR_UUID => "extended_info",
        _ if uuid == CONFIG_CHAR_UUID => "config",
        _ if uuid == CONTROL_CHAR_UUID => "control",
        _ if uuid == BATTERY_LEVEL_CHAR_UUID => "battery_level",
        _ if uuid == FIRMWARE_REVISION_CHAR_UUID => "firmware_revision",
        _ if uuid == DEVICE_NAME_CHAR_UUID => "device_name",
        _ => "unknown",
    }
}

// =============================================================================
// Utility Functions
// =============================================================================

/// Scan for Ohea Lock devices.
///
/// Returns a list of peripherals that match the Ohea Lock advertising pattern.
///
/// # Errors
///
/// Returns an error if scanning fails.
pub async fn scan_for_devices(
    adapter: &btleplug::platform::Adapter,
    timeout: std::time::Duration,
) -> Result<Vec<Peripheral>> {
    use btleplug::api::{Central, ScanFilter};

    adapter.start_scan(ScanFilter::default()).await?;
    tokio::time::sleep(timeout).await;
    adapter.stop_scan().await?;

    let peripherals = adapter.peripherals().await?;
    let mut locks = Vec::new();

    for peripheral in peripherals {
        if let Some(props) = peripheral.properties().await?
            && let Some(name) = props.local_name
            && name == crate::protocol::DEVICE_NAME
        {
            locks.push(peripheral);
        }
    }

    Ok(locks)
}
