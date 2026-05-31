//! Interactive hardware debugging tool for Ohea Lock.
//!
//! This example provides an interactive CLI for debugging and testing
//! Ohea Lock devices. It handles device discovery, pairing, and all
//! lock operations.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example debug --features btleplug-support
//! ```

use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral};
use ohea_lock::{BtleplugTransport, LockState, OheaLock, Result};
use std::io::{self, BufRead, Write};
use std::time::Duration;

// =============================================================================
// Configuration
// =============================================================================

const DEVICE_NAME: &str = "Ohea Lock";
const SCAN_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DebugCommand {
    Lock,
    Unlock,
    Toggle,
    State,
    Info,
    ReadAll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DebugMode {
    Interactive,
    OneShot(DebugCommand),
    Help,
}

fn usage() -> &'static str {
    "Usage: debug [COMMAND]\n\
\n\
Commands:\n\
  lock      Lock the Ohea Lock once and exit\n\
  unlock    Unlock the Ohea Lock once and exit\n\
  toggle    Toggle the current lock state once and exit\n\
  state     Print the current lock state and exit\n\
  info      Print device information and exit\n\
  read-all  Read all known characteristics and exit\n\
\n\
Run without COMMAND to start interactive mode."
}

fn parse_debug_mode_from<I, S>(args: I) -> std::result::Result<DebugMode, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut args = args.into_iter();
    let _program = args.next();
    let Some(command) = args.next() else {
        return Ok(DebugMode::Interactive);
    };

    if args.next().is_some() {
        return Err("Expected at most one command".to_string());
    }

    match command.as_ref().trim().to_ascii_lowercase().as_str() {
        "-h" | "--help" | "help" => Ok(DebugMode::Help),
        "lock" => Ok(DebugMode::OneShot(DebugCommand::Lock)),
        "unlock" => Ok(DebugMode::OneShot(DebugCommand::Unlock)),
        "toggle" => Ok(DebugMode::OneShot(DebugCommand::Toggle)),
        "state" | "status" => Ok(DebugMode::OneShot(DebugCommand::State)),
        "info" => Ok(DebugMode::OneShot(DebugCommand::Info)),
        "read" | "read-all" | "all" => Ok(DebugMode::OneShot(DebugCommand::ReadAll)),
        other => Err(format!("Unknown command: {other}")),
    }
}

// =============================================================================
// Main Entry Point
// =============================================================================

#[tokio::main]
async fn main() -> Result<()> {
    let mode = match parse_debug_mode_from(std::env::args()) {
        Ok(DebugMode::Help) => {
            println!("{}", usage());
            return Ok(());
        }
        Ok(mode) => mode,
        Err(message) => {
            eprintln!("{message}\n\n{}", usage());
            std::process::exit(2);
        }
    };

    println!("\n╔══════════════════════════════════════╗");
    println!("║     Ohea Lock Debug Tool v0.1.0      ║");
    println!("╚══════════════════════════════════════╝\n");

    let adapter = get_adapter().await?;
    let peripheral = match mode {
        DebugMode::Interactive => discover_and_select(&adapter).await?,
        DebugMode::OneShot(_) => discover_one_shot(&adapter).await?,
        DebugMode::Help => unreachable!("help returns before Bluetooth initialization"),
    };
    let lock = connect_to_device(peripheral).await?;

    match mode {
        DebugMode::Interactive => run_interactive_session(lock).await,
        DebugMode::OneShot(command) => run_one_shot_command(lock, command).await,
        DebugMode::Help => unreachable!("help returns before Bluetooth initialization"),
    }
}

// =============================================================================
// Bluetooth Adapter
// =============================================================================

async fn get_adapter() -> Result<Adapter> {
    print!("Initializing Bluetooth adapter... ");
    io::stdout().flush().ok();

    let manager = Manager::new().await?;
    let adapter = manager
        .adapters()
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| ohea_lock::Error::Transport("No Bluetooth adapter found".into()))?;

    println!("✓");
    Ok(adapter)
}

// =============================================================================
// Device Discovery
// =============================================================================

async fn discover_and_select(adapter: &Adapter) -> Result<Peripheral> {
    println!(
        "Scanning for {} devices ({:?})...",
        DEVICE_NAME, SCAN_TIMEOUT
    );

    let devices = scan_ohea_locks(adapter).await?;

    if devices.is_empty() {
        println!("\n⚠ No {} found in scan results.", DEVICE_NAME);
        println!("  Ensure the device is powered on and in range.");
        println!("  If unpaired, the device may need to be in pairing mode.\n");

        if prompt_yes_no("Retry scan?")? {
            return Box::pin(discover_and_select(adapter)).await;
        }
        return Err(ohea_lock::Error::Transport("No device found".into()));
    }

    println!("\nFound {} device(s):\n", devices.len());

    for (i, device) in devices.iter().enumerate() {
        let props = device.properties().await?.unwrap_or_default();
        let name = props.local_name.as_deref().unwrap_or("Unknown");
        let addr = props.address;
        let rssi = props.rssi.map(|r| format!("{r} dBm")).unwrap_or_default();
        println!("  [{}] {} ({}) {}", i + 1, name, addr, rssi);
    }
    println!();

    let selection = if devices.len() == 1 {
        if prompt_yes_no(&format!("Connect to {}?", DEVICE_NAME))? {
            0
        } else {
            return Err(ohea_lock::Error::Transport("User cancelled".into()));
        }
    } else {
        prompt_number("Select device", 1, devices.len())? - 1
    };

    Ok(devices
        .into_iter()
        .nth(selection)
        .expect("selection is in range"))
}

async fn discover_one_shot(adapter: &Adapter) -> Result<Peripheral> {
    println!(
        "Scanning for {} devices ({:?})...",
        DEVICE_NAME, SCAN_TIMEOUT
    );

    let mut devices = scan_ohea_locks(adapter).await?;
    match devices.len() {
        0 => Err(ohea_lock::Error::Transport("No device found".into())),
        1 => Ok(devices.remove(0)),
        _ => Err(ohea_lock::Error::Transport(
            "Multiple Ohea Lock devices found; use interactive mode".into(),
        )),
    }
}

async fn scan_ohea_locks(adapter: &Adapter) -> Result<Vec<Peripheral>> {
    adapter.start_scan(ScanFilter::default()).await?;
    tokio::time::sleep(SCAN_TIMEOUT).await;
    adapter.stop_scan().await?;
    find_ohea_locks(adapter).await
}

async fn find_ohea_locks(adapter: &Adapter) -> Result<Vec<Peripheral>> {
    let peripherals = adapter.peripherals().await?;
    let mut locks = Vec::new();

    for peripheral in peripherals {
        if let Some(props) = peripheral.properties().await?
            && props.local_name.as_deref() == Some(DEVICE_NAME)
        {
            locks.push(peripheral);
        }
    }

    Ok(locks)
}

// =============================================================================
// Service & Characteristic Discovery Display
// =============================================================================

async fn display_services_and_characteristics(peripheral: &Peripheral) -> Result<()> {
    use btleplug::api::CharPropFlags;

    let services = peripheral.services();

    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│ Discovered Services & Characteristics                       │");
    println!("└─────────────────────────────────────────────────────────────┘");

    for service in services {
        println!("\n📦 Service: {}", service.uuid);
        println!("   Primary: {}", if service.primary { "Yes" } else { "No" });

        if service.characteristics.is_empty() {
            println!("   └─ (no characteristics)");
        } else {
            for (idx, characteristic) in service.characteristics.iter().enumerate() {
                let is_last = idx == service.characteristics.len() - 1;
                let prefix = if is_last { "└─" } else { "├─" };

                println!("   {} 🔧 {}", prefix, characteristic.uuid);

                let props = characteristic.properties;
                let mut properties = Vec::new();
                if props.contains(CharPropFlags::READ) {
                    properties.push("Read");
                }
                if props.contains(CharPropFlags::WRITE) {
                    properties.push("Write");
                }
                if props.contains(CharPropFlags::WRITE_WITHOUT_RESPONSE) {
                    properties.push("WriteNoResp");
                }
                if props.contains(CharPropFlags::NOTIFY) {
                    properties.push("Notify");
                }
                if props.contains(CharPropFlags::INDICATE) {
                    properties.push("Indicate");
                }
                if props.contains(CharPropFlags::BROADCAST) {
                    properties.push("Broadcast");
                }

                if !properties.is_empty() {
                    let indent = if is_last { "      " } else { "   │  " };
                    println!("{}Properties: {}", indent, properties.join(", "));
                }

                // Display descriptors if any
                if !characteristic.descriptors.is_empty() {
                    let indent = if is_last { "      " } else { "   │  " };
                    println!(
                        "{}Descriptors: {} found",
                        indent,
                        characteristic.descriptors.len()
                    );
                    for descriptor in &characteristic.descriptors {
                        println!("{}  • {}", indent, descriptor.uuid);
                    }
                }
            }
        }
    }

    println!("\n─────────────────────────────────────────────────────────────\n");
    Ok(())
}

// =============================================================================
// Connection & Pairing
// =============================================================================

async fn connect_to_device(peripheral: Peripheral) -> Result<OheaLock<BtleplugTransport>> {
    let is_connected = peripheral.is_connected().await?;

    if is_connected {
        println!("Device already connected, discovering services...");
    } else {
        print!("Connecting... ");
        io::stdout().flush().ok();
        peripheral.connect().await?;
        println!("✓");

        print!("Discovering services (may trigger pairing)... ");
        io::stdout().flush().ok();
    }

    peripheral.discover_services().await?;
    println!("✓\n");

    // Display discovered services and characteristics
    display_services_and_characteristics(&peripheral).await?;

    let transport = BtleplugTransport::new(peripheral).await?;
    let mut lock = OheaLock::new(transport);

    print!("Initializing session... ");
    io::stdout().flush().ok();
    lock.initialize().await?;
    println!("✓\n");

    // Display device info
    display_device_info(&lock).await?;

    Ok(lock)
}

async fn display_device_info(lock: &OheaLock<BtleplugTransport>) -> Result<()> {
    let info = lock.get_device_info().await?;

    println!("┌─────────────────────────────────────┐");
    println!("│ Device Information                  │");
    println!("├─────────────────────────────────────┤");
    println!("│ Name:     {:<25} │", info.name);
    println!("│ Firmware: {:<25} │", info.firmware_version);
    println!("│ Battery:  {:<25} │", format!("{}%", info.battery_level));
    println!("└─────────────────────────────────────┘");
    println!();

    Ok(())
}

// =============================================================================
// Interactive Session
// =============================================================================

async fn run_one_shot_command(
    lock: OheaLock<BtleplugTransport>,
    command: DebugCommand,
) -> Result<()> {
    match command {
        DebugCommand::Lock => {
            print!("Locking... ");
            io::stdout().flush().ok();
            lock.lock().await?;
            tokio::time::sleep(Duration::from_millis(300)).await;
            println!("✓");
        }
        DebugCommand::Unlock => {
            print!("Unlocking... ");
            io::stdout().flush().ok();
            lock.unlock().await?;
            tokio::time::sleep(Duration::from_millis(300)).await;
            println!("✓");
        }
        DebugCommand::Toggle => {
            let current = lock.get_lock_state().await?;
            if current.is_locked() {
                print!("Unlocking... ");
                io::stdout().flush().ok();
                lock.unlock().await?;
            } else {
                print!("Locking... ");
                io::stdout().flush().ok();
                lock.lock().await?;
            }
            tokio::time::sleep(Duration::from_millis(300)).await;
            let state = lock.get_lock_state().await?;
            println!("✓ {:?}", state);
        }
        DebugCommand::State => {
            let state = lock.get_lock_state().await?;
            println!("Lock state: {:?}", state);
        }
        DebugCommand::Info => {
            display_device_info(&lock).await?;
        }
        DebugCommand::ReadAll => {
            read_all_characteristics(&lock).await?;
        }
    }

    Ok(())
}

async fn run_interactive_session(lock: OheaLock<BtleplugTransport>) -> Result<()> {
    loop {
        let state = lock.get_lock_state().await?;
        let state_icon = match state {
            LockState::Locked => "🔒",
            LockState::Unlocked => "🔓",
        };

        println!("\nCurrent state: {} {:?}", state_icon, state);
        println!();
        println!("Commands:");
        println!("  [1] Lock");
        println!("  [2] Unlock");
        println!("  [3] Toggle");
        println!("  [4] Refresh state");
        println!("  [5] Show device info");
        println!("  [6] Read all characteristics");
        println!("  [q] Quit");
        println!();

        let input = prompt_string("Enter command")?;

        match input.trim().to_lowercase().as_str() {
            "1" | "lock" => {
                print!("Locking... ");
                io::stdout().flush().ok();
                lock.lock().await?;
                tokio::time::sleep(Duration::from_millis(300)).await;
                println!("✓");
            }
            "2" | "unlock" => {
                print!("Unlocking... ");
                io::stdout().flush().ok();
                lock.unlock().await?;
                tokio::time::sleep(Duration::from_millis(300)).await;
                println!("✓");
            }
            "3" | "toggle" => {
                let current = lock.get_lock_state().await?;
                let action = if current.is_locked() {
                    "Unlocking"
                } else {
                    "Locking"
                };
                print!("{}... ", action);
                io::stdout().flush().ok();

                if current.is_locked() {
                    lock.unlock().await?;
                } else {
                    lock.lock().await?;
                }
                tokio::time::sleep(Duration::from_millis(300)).await;
                println!("✓");
            }
            "4" | "refresh" | "r" => {
                println!("Refreshing...");
            }
            "5" | "info" | "i" => {
                display_device_info(&lock).await?;
            }
            "6" | "read" | "all" => {
                read_all_characteristics(&lock).await?;
            }
            "q" | "quit" | "exit" => {
                println!("Disconnecting...");
                break;
            }
            "" => continue,
            _ => {
                println!("Unknown command: {}", input.trim());
            }
        }
    }

    println!("Goodbye!\n");
    Ok(())
}

async fn read_all_characteristics(lock: &OheaLock<BtleplugTransport>) -> Result<()> {
    println!("\n┌─────────────────────────────────────┐");
    println!("│ All Readable Characteristics        │");
    println!("├─────────────────────────────────────┤");

    // Lock state
    match lock.get_lock_state().await {
        Ok(state) => println!(
            "│ Lock State:    {:?}{} │",
            state,
            " ".repeat(14 - format!("{:?}", state).len())
        ),
        Err(e) => println!(
            "│ Lock State:    Error: {:<12} │",
            e.to_string().chars().take(12).collect::<String>()
        ),
    }

    // Lock position
    match lock.get_lock_position().await {
        Ok(pos) => println!("│ Lock Position: {:#04x}{} │", pos, " ".repeat(17)),
        Err(e) => println!(
            "│ Lock Position: Error: {:<12} │",
            e.to_string().chars().take(12).collect::<String>()
        ),
    }

    // Battery
    match lock.get_battery_level().await {
        Ok(level) => println!(
            "│ Battery:       {}%{} │",
            level,
            " ".repeat(if level < 10 {
                19
            } else if level < 100 {
                18
            } else {
                17
            })
        ),
        Err(e) => println!(
            "│ Battery:       Error: {:<12} │",
            e.to_string().chars().take(12).collect::<String>()
        ),
    }

    // Firmware
    match lock.get_firmware_version().await {
        Ok(ver) => println!("│ Firmware:      {:<21} │", ver),
        Err(e) => println!(
            "│ Firmware:      Error: {:<12} │",
            e.to_string().chars().take(12).collect::<String>()
        ),
    }

    // Device name
    match lock.get_device_name().await {
        Ok(name) => println!("│ Device Name:   {:<21} │", name),
        Err(e) => println!(
            "│ Device Name:   Error: {:<12} │",
            e.to_string().chars().take(12).collect::<String>()
        ),
    }

    println!("└─────────────────────────────────────┘");
    Ok(())
}

// =============================================================================
// User Input Helpers
// =============================================================================

fn prompt_yes_no(question: &str) -> io::Result<bool> {
    print!("{} [y/N]: ", question);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().lock().read_line(&mut input)?;

    Ok(matches!(input.trim().to_lowercase().as_str(), "y" | "yes"))
}

fn prompt_number(prompt: &str, min: usize, max: usize) -> io::Result<usize> {
    loop {
        print!("{} [{}-{}]: ", prompt, min, max);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().lock().read_line(&mut input)?;

        if let Ok(n) = input.trim().parse::<usize>()
            && n >= min
            && n <= max
        {
            return Ok(n);
        }
        println!("Please enter a number between {} and {}", min, max);
    }
}

fn prompt_string(prompt: &str) -> io::Result<String> {
    print!("{}: ", prompt);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().lock().read_line(&mut input)?;
    Ok(input)
}

#[cfg(test)]
mod tests {
    use super::{DebugCommand, DebugMode, parse_debug_mode_from};

    #[test]
    fn parse_no_args_defaults_to_interactive() {
        assert_eq!(
            parse_debug_mode_from(["debug"]).unwrap(),
            DebugMode::Interactive
        );
    }

    #[test]
    fn parse_supported_one_shot_commands() {
        assert_eq!(
            parse_debug_mode_from(["debug", "lock"]).unwrap(),
            DebugMode::OneShot(DebugCommand::Lock)
        );
        assert_eq!(
            parse_debug_mode_from(["debug", "unlock"]).unwrap(),
            DebugMode::OneShot(DebugCommand::Unlock)
        );
        assert_eq!(
            parse_debug_mode_from(["debug", "toggle"]).unwrap(),
            DebugMode::OneShot(DebugCommand::Toggle)
        );
        assert_eq!(
            parse_debug_mode_from(["debug", "state"]).unwrap(),
            DebugMode::OneShot(DebugCommand::State)
        );
        assert_eq!(
            parse_debug_mode_from(["debug", "info"]).unwrap(),
            DebugMode::OneShot(DebugCommand::Info)
        );
        assert_eq!(
            parse_debug_mode_from(["debug", "read-all"]).unwrap(),
            DebugMode::OneShot(DebugCommand::ReadAll)
        );
    }

    #[test]
    fn parse_aliases() {
        assert_eq!(
            parse_debug_mode_from(["debug", "read"]).unwrap(),
            DebugMode::OneShot(DebugCommand::ReadAll)
        );
        assert_eq!(
            parse_debug_mode_from(["debug", "status"]).unwrap(),
            DebugMode::OneShot(DebugCommand::State)
        );
        assert_eq!(
            parse_debug_mode_from(["debug", "-h"]).unwrap(),
            DebugMode::Help
        );
        assert_eq!(
            parse_debug_mode_from(["debug", "--help"]).unwrap(),
            DebugMode::Help
        );
    }

    #[test]
    fn parse_rejects_unknown_command() {
        let err = parse_debug_mode_from(["debug", "open"]).unwrap_err();
        assert!(err.contains("Unknown command: open"));
    }

    #[test]
    fn parse_rejects_extra_arguments() {
        let err = parse_debug_mode_from(["debug", "lock", "extra"]).unwrap_err();
        assert!(err.contains("Expected at most one command"));
    }
}
