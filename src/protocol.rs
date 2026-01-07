//! Protocol constants and types for Ohea Lock BLE communication.
//!
//! This module contains all the UUIDs, handles, and protocol-specific types
//! derived from packet capture analysis.

use uuid::Uuid;

// =============================================================================
// Bluetooth Base UUID
// =============================================================================

/// Bluetooth Base UUID: 00000000-0000-1000-8000-00805F9B34FB
/// Used to expand 16-bit UUIDs to 128-bit.
const BLUETOOTH_BASE_UUID: u128 = 0x0000_0000_0000_1000_8000_0080_5F9B_34FB;

/// Convert a 16-bit BLE UUID to a full 128-bit UUID.
#[must_use]
const fn uuid_from_u16(short: u16) -> Uuid {
    Uuid::from_u128(BLUETOOTH_BASE_UUID | ((short as u128) << 96))
}

// =============================================================================
// Service UUIDs
// =============================================================================

/// Custom Ohea Lock service UUID.
/// Handles: 0x0025-0x003F
pub const LOCK_SERVICE_UUID: Uuid = Uuid::from_u128(0x0A0F0001_0000_1000_8000_00805F9B34FB);

/// Standard Battery Service UUID (0x180F).
pub const BATTERY_SERVICE_UUID: Uuid = uuid_from_u16(0x180F);

/// Standard Device Information Service UUID (0x180A).
pub const DEVICE_INFO_SERVICE_UUID: Uuid = uuid_from_u16(0x180A);

/// Standard Generic Access Service UUID (0x1800).
pub const GENERIC_ACCESS_SERVICE_UUID: Uuid = uuid_from_u16(0x1800);

/// Dialog Semiconductor OTA Service UUID (0xFEF5).
pub const DIALOG_OTA_SERVICE_UUID: Uuid = uuid_from_u16(0xFEF5);

// =============================================================================
// Lock Service Characteristic UUIDs
// =============================================================================

/// Lock state characteristic - Read/Write/Notify.
/// Handle: 0x0027
/// Values: 0x00 = locked, 0x01 = unlocked
pub const LOCK_STATE_CHAR_UUID: Uuid = Uuid::from_u128(0x0A0F0001_0000_1000_8000_00805F9B34FB);

/// Status notification characteristic - Notify.
/// Handle: 0x002B
pub const STATUS_CHAR_UUID: Uuid = Uuid::from_u128(0x0A0F0011_0000_1000_8000_00805F9B34FB);

/// Lock position characteristic - Read.
/// Handle: 0x002F
pub const LOCK_POSITION_CHAR_UUID: Uuid = Uuid::from_u128(0x0A0F0002_0000_1000_8000_00805F9B34FB);

/// Command register characteristic - Write.
/// Handle: 0x0032
pub const COMMAND_CHAR_UUID: Uuid = Uuid::from_u128(0x0A0F0003_0000_1000_8000_00805F9B34FB);

/// Device info characteristic - Read.
/// Handle: 0x0035
pub const DEVICE_INFO_CHAR_UUID: Uuid = Uuid::from_u128(0x0A0F0004_0000_1000_8000_00805F9B34FB);

/// Extended info characteristic - Read.
/// Handle: 0x0038
pub const EXTENDED_INFO_CHAR_UUID: Uuid = Uuid::from_u128(0x0A0F1004_0000_1000_8000_00805F9B34FB);

/// Configuration characteristic - Write.
/// Handle: 0x003B
pub const CONFIG_CHAR_UUID: Uuid = Uuid::from_u128(0x0A0F0005_0000_1000_8000_00805F9B34FB);

/// Additional control characteristic - Write.
/// Handle: 0x003E
pub const CONTROL_CHAR_UUID: Uuid = Uuid::from_u128(0x0A0F0006_0000_1000_8000_00805F9B34FB);

// =============================================================================
// Standard Characteristic UUIDs
// =============================================================================

/// Battery Level characteristic UUID (0x2A19).
pub const BATTERY_LEVEL_CHAR_UUID: Uuid = uuid_from_u16(0x2A19);

/// Firmware Revision String characteristic UUID (0x2A26).
pub const FIRMWARE_REVISION_CHAR_UUID: Uuid = uuid_from_u16(0x2A26);

/// Device Name characteristic UUID (0x2A00).
pub const DEVICE_NAME_CHAR_UUID: Uuid = uuid_from_u16(0x2A00);

// =============================================================================
// Protocol Types
// =============================================================================

/// Lock state enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum LockState {
    /// Lock is in locked position.
    Locked = 0x00,
    /// Lock is in unlocked position.
    Unlocked = 0x01,
}

impl LockState {
    /// Create a `LockState` from a raw byte value.
    #[must_use]
    pub const fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x00 => Some(Self::Locked),
            0x01 => Some(Self::Unlocked),
            _ => None,
        }
    }

    /// Convert to raw byte value.
    #[must_use]
    pub const fn as_byte(self) -> u8 {
        self as u8
    }

    /// Returns `true` if the lock is locked.
    #[must_use]
    pub const fn is_locked(self) -> bool {
        matches!(self, Self::Locked)
    }

    /// Returns `true` if the lock is unlocked.
    #[must_use]
    pub const fn is_unlocked(self) -> bool {
        matches!(self, Self::Unlocked)
    }
}

impl TryFrom<u8> for LockState {
    type Error = crate::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::from_byte(value)
            .ok_or_else(|| crate::Error::InvalidResponse(format!("invalid lock state: {value:#04x}")))
    }
}

impl From<LockState> for u8 {
    fn from(state: LockState) -> Self {
        state.as_byte()
    }
}

/// Device information retrieved from the lock.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    /// Device name (e.g., "Ohea Lock").
    pub name: String,
    /// Firmware revision string (e.g., "1.0").
    pub firmware_version: String,
    /// Battery level as percentage (0-100).
    pub battery_level: u8,
}

/// Manufacturer ID found in advertising data.
pub const MANUFACTURER_ID: u16 = 0x0A0F;

/// Device name as advertised.
pub const DEVICE_NAME: &str = "Ohea Lock";

// =============================================================================
// Command Builders
// =============================================================================

/// Build an initialization command for the command register.
///
/// This command is sent after pairing to initialize the session.
/// Observed format: `[0x1A, 0x01, 0x04, 0x00, 0x31]`
#[must_use]
pub fn build_init_command() -> [u8; 5] {
    [0x1A, 0x01, 0x04, 0x00, 0x31]
}

// =============================================================================
// ATT Error Codes
// =============================================================================

/// ATT Error code for Insufficient Authentication.
pub const ATT_ERR_INSUFFICIENT_AUTH: u8 = 0x05;

/// ATT Error code for Attribute Not Found.
pub const ATT_ERR_ATTR_NOT_FOUND: u8 = 0x0A;

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // UUID Construction Tests
    // =========================================================================

    #[test]
    fn uuid_from_u16_constructs_correct_bluetooth_uuid() {
        // Battery Level 0x2A19 -> 00002A19-0000-1000-8000-00805F9B34FB
        let expected = Uuid::parse_str("00002A19-0000-1000-8000-00805F9B34FB").unwrap();
        assert_eq!(uuid_from_u16(0x2A19), expected);
    }

    #[test]
    fn standard_service_uuids_are_correct() {
        assert_eq!(
            BATTERY_SERVICE_UUID,
            Uuid::parse_str("0000180F-0000-1000-8000-00805F9B34FB").unwrap()
        );
        assert_eq!(
            DEVICE_INFO_SERVICE_UUID,
            Uuid::parse_str("0000180A-0000-1000-8000-00805F9B34FB").unwrap()
        );
        assert_eq!(
            GENERIC_ACCESS_SERVICE_UUID,
            Uuid::parse_str("00001800-0000-1000-8000-00805F9B34FB").unwrap()
        );
    }

    #[test]
    fn lock_service_uuid_matches_manufacturer_spec() {
        assert_eq!(
            LOCK_SERVICE_UUID,
            Uuid::parse_str("0A0F0001-0000-1000-8000-00805F9B34FB").unwrap()
        );
    }

    #[test]
    fn lock_characteristic_uuids_are_correct() {
        let cases = [
            (LOCK_STATE_CHAR_UUID, "0A0F0001-0000-1000-8000-00805F9B34FB"),
            (STATUS_CHAR_UUID, "0A0F0011-0000-1000-8000-00805F9B34FB"),
            (LOCK_POSITION_CHAR_UUID, "0A0F0002-0000-1000-8000-00805F9B34FB"),
            (COMMAND_CHAR_UUID, "0A0F0003-0000-1000-8000-00805F9B34FB"),
            (DEVICE_INFO_CHAR_UUID, "0A0F0004-0000-1000-8000-00805F9B34FB"),
            (EXTENDED_INFO_CHAR_UUID, "0A0F1004-0000-1000-8000-00805F9B34FB"),
            (CONFIG_CHAR_UUID, "0A0F0005-0000-1000-8000-00805F9B34FB"),
            (CONTROL_CHAR_UUID, "0A0F0006-0000-1000-8000-00805F9B34FB"),
        ];
        for (uuid, expected) in cases {
            assert_eq!(uuid, Uuid::parse_str(expected).unwrap(), "UUID mismatch for {expected}");
        }
    }

    #[test]
    fn standard_characteristic_uuids_are_correct() {
        assert_eq!(
            BATTERY_LEVEL_CHAR_UUID,
            Uuid::parse_str("00002A19-0000-1000-8000-00805F9B34FB").unwrap()
        );
        assert_eq!(
            FIRMWARE_REVISION_CHAR_UUID,
            Uuid::parse_str("00002A26-0000-1000-8000-00805F9B34FB").unwrap()
        );
        assert_eq!(
            DEVICE_NAME_CHAR_UUID,
            Uuid::parse_str("00002A00-0000-1000-8000-00805F9B34FB").unwrap()
        );
    }

    // =========================================================================
    // LockState Tests
    // =========================================================================

    #[test]
    fn lock_state_from_byte_valid_values() {
        assert_eq!(LockState::from_byte(0x00), Some(LockState::Locked));
        assert_eq!(LockState::from_byte(0x01), Some(LockState::Unlocked));
    }

    #[test]
    fn lock_state_from_byte_invalid_values() {
        (0x02..=0xFF).for_each(|b| assert_eq!(LockState::from_byte(b), None));
    }

    #[test]
    fn lock_state_as_byte_roundtrip() {
        [LockState::Locked, LockState::Unlocked]
            .into_iter()
            .for_each(|s| assert_eq!(LockState::from_byte(s.as_byte()), Some(s)));
    }

    #[test]
    fn lock_state_predicates() {
        assert!(LockState::Locked.is_locked());
        assert!(!LockState::Locked.is_unlocked());
        assert!(!LockState::Unlocked.is_locked());
        assert!(LockState::Unlocked.is_unlocked());
    }

    #[test]
    fn lock_state_try_from_valid() {
        assert_eq!(LockState::try_from(0x00).unwrap(), LockState::Locked);
        assert_eq!(LockState::try_from(0x01).unwrap(), LockState::Unlocked);
    }

    #[test]
    fn lock_state_try_from_invalid() {
        assert!(LockState::try_from(0x02).is_err());
        assert!(LockState::try_from(0xFF).is_err());
    }

    #[test]
    fn lock_state_into_u8() {
        assert_eq!(u8::from(LockState::Locked), 0x00);
        assert_eq!(u8::from(LockState::Unlocked), 0x01);
    }

    // =========================================================================
    // DeviceInfo Tests
    // =========================================================================

    #[test]
    fn device_info_equality() {
        let a = DeviceInfo {
            name: "Ohea Lock".to_string(),
            firmware_version: "1.0".to_string(),
            battery_level: 100,
        };
        let b = a.clone();
        assert_eq!(a, b);
    }

    // =========================================================================
    // Constants Tests
    // =========================================================================

    #[test]
    fn device_name_matches_packet_capture() {
        // From packet capture: "Ohea Lock" = 4F 68 65 61 20 4C 6F 63 6B
        assert_eq!(DEVICE_NAME, "Ohea Lock");
        assert_eq!(DEVICE_NAME.as_bytes(), &[0x4F, 0x68, 0x65, 0x61, 0x20, 0x4C, 0x6F, 0x63, 0x6B]);
    }

    #[test]
    fn manufacturer_id_matches_packet_capture() {
        assert_eq!(MANUFACTURER_ID, 0x0A0F);
    }

    // =========================================================================
    // Command Builder Tests
    // =========================================================================

    #[test]
    fn init_command_matches_packet_capture() {
        // From packet capture: 1A 01 04 00 31
        assert_eq!(build_init_command(), [0x1A, 0x01, 0x04, 0x00, 0x31]);
    }

    #[test]
    fn init_command_length_is_five_bytes() {
        assert_eq!(build_init_command().len(), 5);
    }

    // =========================================================================
    // ATT Error Codes Tests
    // =========================================================================

    #[test]
    fn att_error_codes_match_bluetooth_spec() {
        assert_eq!(ATT_ERR_INSUFFICIENT_AUTH, 0x05);
        assert_eq!(ATT_ERR_ATTR_NOT_FOUND, 0x0A);
    }
}
