# Ohea Lock

A Rust library for controlling the [Ohea Lock](https://parts.lixil.co.jp/lixilps/shop/campaign/ohealock)(おへやろっく) BLE smart lock from LIXIL Corporation.

## Features

- Transport-agnostic core design
- Optional `btleplug` support for desktop platforms
- Async-first API
- Full lock control (lock, unlock, state reading)
- Device information retrieval (battery, firmware, name)

## Installation

```toml
[dependencies]
ohea-lock = { version = "0.1", features = ["btleplug-support"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

## Example

Complete example: discover → connect → pair → control lock

```rust
use ohea_lock::{BtleplugTransport, OheaLock, Result};
use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::Manager;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Get Bluetooth adapter
    let manager = Manager::new().await?;
    let adapter = manager
        .adapters()
        .await?
        .into_iter()
        .next()
        .expect("No Bluetooth adapter found");

    // 2. Scan for devices
    println!("Scanning for Ohea Lock...");
    adapter.start_scan(ScanFilter::default()).await?;
    tokio::time::sleep(Duration::from_secs(5)).await;
    adapter.stop_scan().await?;

    // 3. Find Ohea Lock device
    let peripheral = adapter
        .peripherals()
        .await?
        .into_iter()
        .filter(|p| {
            futures::executor::block_on(async {
                p.properties()
                    .await
                    .ok()
                    .flatten()
                    .and_then(|props| props.local_name)
                    .is_some_and(|name| name == "Ohea Lock")
            })
        })
        .next()
        .expect("Ohea Lock not found");

    // 4. Connect and discover services (triggers pairing on first connection)
    println!("Connecting...");
    let transport = BtleplugTransport::connect_and_discover(peripheral).await?;
    let mut lock = OheaLock::new(transport);

    // 5. Initialize session
    lock.initialize().await?;

    // 6. Get device information
    let info = lock.get_device_info().await?;
    println!("Device: {}", info.name);
    println!("Firmware: {}", info.firmware_version);
    println!("Battery: {}%", info.battery_level);

    // 7. Read lock state
    let state = lock.get_lock_state().await?;
    println!("Lock state: {:?}", state);

    // 8. Control the lock
    println!("Unlocking...");
    lock.unlock().await?;
    tokio::time::sleep(Duration::from_secs(2)).await;

    println!("Locking...");
    lock.lock().await?;

    println!("Done!");
    Ok(())
}
```

## API Overview

```rust
// Create lock instance
let lock = OheaLock::new(transport);

// Initialize (required after first connection)
lock.initialize().await?;

// Lock control
lock.lock().await?;
lock.unlock().await?;
let state = lock.get_lock_state().await?;  // LockState::Locked | LockState::Unlocked

// Device info
let info = lock.get_device_info().await?;  // DeviceInfo { name, firmware_version, battery_level }
let battery = lock.get_battery_level().await?;  // u8 (0-100)
let firmware = lock.get_firmware_version().await?;  // String

// Notifications
lock.subscribe_lock_state().await?;
lock.subscribe_battery_level().await?;
lock.unsubscribe_all().await?;
```

## Testing

```bash
# Unit tests
cargo test --features btleplug-support
```

## Debug Tool

For hardware debugging and testing:

```bash
cargo run --example debug --features btleplug-support
```

Run a single command and exit:

```bash
cargo run --example debug --features btleplug-support -- state
cargo run --example debug --features btleplug-support -- lock
cargo run --example debug --features btleplug-support -- unlock
cargo run --example debug --features btleplug-support -- toggle
cargo run --example debug --features btleplug-support -- info
cargo run --example debug --features btleplug-support -- read-all
```

Install the debug example as a local command:

```bash
scripts/install-debug-bin.sh
/Users/vvx/.local/bin/ohea-lock-debug state
```

## License

GPL-3.0
