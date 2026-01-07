//! Transport trait abstraction for BLE communication.
//!
//! This module defines the core trait that abstracts over different BLE backends.
//! Implementations can be provided for btleplug, embedded BLE stacks, or mock transports.

use async_trait::async_trait;
use uuid::Uuid;

use crate::error::Result;

/// A trait abstracting BLE GATT operations.
///
/// This trait allows the core protocol logic to be independent of the underlying
/// BLE implementation. Implementations handle the actual read/write/notify operations.
#[async_trait]
pub trait Transport: Send + Sync {
    /// Read a characteristic value by its UUID.
    ///
    /// # Errors
    ///
    /// Returns an error if the characteristic is not found or the read fails.
    async fn read(&self, char_uuid: Uuid) -> Result<Vec<u8>>;

    /// Write a value to a characteristic by its UUID.
    ///
    /// # Errors
    ///
    /// Returns an error if the characteristic is not found or the write fails.
    async fn write(&self, char_uuid: Uuid, data: &[u8]) -> Result<()>;

    /// Write a value to a characteristic without waiting for a response.
    ///
    /// # Errors
    ///
    /// Returns an error if the characteristic is not found or the write fails.
    async fn write_without_response(&self, char_uuid: Uuid, data: &[u8]) -> Result<()>;

    /// Subscribe to notifications from a characteristic.
    ///
    /// # Errors
    ///
    /// Returns an error if the characteristic is not found or subscription fails.
    async fn subscribe(&self, char_uuid: Uuid) -> Result<()>;

    /// Unsubscribe from notifications from a characteristic.
    ///
    /// # Errors
    ///
    /// Returns an error if the characteristic is not found or unsubscription fails.
    async fn unsubscribe(&self, char_uuid: Uuid) -> Result<()>;

    /// Check if the transport is currently connected.
    fn is_connected(&self) -> bool;
}

/// Extension trait providing high-level lock operations on any transport.
#[async_trait]
pub trait TransportExt: Transport {
    /// Read a string value from a characteristic.
    ///
    /// # Errors
    ///
    /// Returns an error if the read fails or the data is not valid UTF-8.
    async fn read_string(&self, char_uuid: Uuid) -> Result<String> {
        let data = self.read(char_uuid).await?;
        String::from_utf8(data)
            .map_err(|e| crate::Error::InvalidResponse(format!("invalid UTF-8: {e}")))
    }

    /// Read a single byte value from a characteristic.
    ///
    /// # Errors
    ///
    /// Returns an error if the read fails or the response is empty.
    async fn read_byte(&self, char_uuid: Uuid) -> Result<u8> {
        let data = self.read(char_uuid).await?;
        data.first()
            .copied()
            .ok_or_else(|| crate::Error::InvalidResponse("empty response".to_string()))
    }
}

// Blanket implementation for all Transport implementors
impl<T: Transport + ?Sized> TransportExt for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    /// Mock transport for testing without hardware.
    #[derive(Default)]
    struct MockTransport {
        responses: Arc<RwLock<HashMap<Uuid, Vec<u8>>>>,
        writes: Arc<RwLock<Vec<(Uuid, Vec<u8>)>>>,
        subscribed: Arc<RwLock<Vec<Uuid>>>,
        connected: bool,
    }

    impl MockTransport {
        fn new() -> Self {
            Self {
                connected: true,
                ..Default::default()
            }
        }

        async fn set_response(&self, uuid: Uuid, data: Vec<u8>) {
            self.responses.write().await.insert(uuid, data);
        }

        async fn get_writes(&self) -> Vec<(Uuid, Vec<u8>)> {
            self.writes.read().await.clone()
        }

        async fn get_subscribed(&self) -> Vec<Uuid> {
            self.subscribed.read().await.clone()
        }
    }

    #[async_trait]
    impl Transport for MockTransport {
        async fn read(&self, char_uuid: Uuid) -> Result<Vec<u8>> {
            self.responses
                .read()
                .await
                .get(&char_uuid)
                .cloned()
                .ok_or_else(|| crate::Error::CharacteristicNotFound("mock"))
        }

        async fn write(&self, char_uuid: Uuid, data: &[u8]) -> Result<()> {
            self.writes.write().await.push((char_uuid, data.to_vec()));
            Ok(())
        }

        async fn write_without_response(&self, char_uuid: Uuid, data: &[u8]) -> Result<()> {
            self.write(char_uuid, data).await
        }

        async fn subscribe(&self, char_uuid: Uuid) -> Result<()> {
            self.subscribed.write().await.push(char_uuid);
            Ok(())
        }

        async fn unsubscribe(&self, char_uuid: Uuid) -> Result<()> {
            self.subscribed.write().await.retain(|&u| u != char_uuid);
            Ok(())
        }

        fn is_connected(&self) -> bool {
            self.connected
        }
    }

    // =========================================================================
    // Test UUIDs - using fixed UUIDs instead of random for reproducibility
    // =========================================================================
    
    const TEST_UUID_1: Uuid = Uuid::from_u128(0x12345678_1234_1234_1234_123456789ABC);
    const TEST_UUID_2: Uuid = Uuid::from_u128(0x87654321_4321_4321_4321_CBA987654321);

    // =========================================================================
    // TransportExt Tests
    // =========================================================================

    #[tokio::test]
    async fn read_string_parses_utf8() {
        let transport = MockTransport::new();
        transport.set_response(TEST_UUID_1, b"Ohea Lock".to_vec()).await;

        let result = transport.read_string(TEST_UUID_1).await.unwrap();
        assert_eq!(result, "Ohea Lock");
    }

    #[tokio::test]
    async fn read_string_errors_on_invalid_utf8() {
        let transport = MockTransport::new();
        transport.set_response(TEST_UUID_1, vec![0xFF, 0xFE]).await;

        let result = transport.read_string(TEST_UUID_1).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("UTF-8"));
    }

    #[tokio::test]
    async fn read_byte_returns_first_byte() {
        let transport = MockTransport::new();
        transport.set_response(TEST_UUID_1, vec![0x64, 0x00, 0x00]).await;

        let result = transport.read_byte(TEST_UUID_1).await.unwrap();
        assert_eq!(result, 0x64);
    }

    #[tokio::test]
    async fn read_byte_errors_on_empty_response() {
        let transport = MockTransport::new();
        transport.set_response(TEST_UUID_1, vec![]).await;

        let result = transport.read_byte(TEST_UUID_1).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[tokio::test]
    async fn write_records_data() {
        let transport = MockTransport::new();
        let data = vec![0x01, 0x02, 0x03];

        transport.write(TEST_UUID_1, &data).await.unwrap();

        let writes = transport.get_writes().await;
        assert_eq!(writes.len(), 1);
        assert_eq!(writes[0], (TEST_UUID_1, data));
    }

    #[tokio::test]
    async fn subscribe_and_unsubscribe() {
        let transport = MockTransport::new();

        transport.subscribe(TEST_UUID_1).await.unwrap();
        assert!(transport.get_subscribed().await.contains(&TEST_UUID_1));

        transport.unsubscribe(TEST_UUID_1).await.unwrap();
        assert!(!transport.get_subscribed().await.contains(&TEST_UUID_1));
    }

    #[tokio::test]
    async fn read_errors_when_characteristic_not_found() {
        let transport = MockTransport::new();

        let result = transport.read(TEST_UUID_2).await;
        assert!(result.is_err());
    }
}
