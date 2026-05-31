# Subtask 01: `examples/debug.rs`

> **Scope:** This worker owns only `examples/debug.rs`. Do not edit README, installer scripts, or global skill files.

**Goal:** Keep the existing interactive debug tool as the default and add one-shot subcommands for `lock`, `unlock`, `toggle`, `state`, `info`, and `read-all`.

**Behavior Contract:**

- No argument starts the current interactive mode.
- `--help`, `-h`, and `help` print usage and exit before Bluetooth initialization.
- One-shot mode scans for Ohea Lock devices, refuses zero or multiple devices, connects to exactly one device, initializes, runs one command, prints the result, and exits.
- One-shot multiple-device error must be `Error::Transport("Multiple Ohea Lock devices found; use interactive mode")`.

## Steps

- [ ] **Step 1: Add parser tests at the bottom of `examples/debug.rs`**

Append after `prompt_string`:

```rust
#[cfg(test)]
mod tests {
    use super::{parse_debug_mode_from, DebugCommand, DebugMode};

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
        assert_eq!(parse_debug_mode_from(["debug", "-h"]).unwrap(), DebugMode::Help);
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
```

- [ ] **Step 2: Run parser tests and confirm RED**

Run:

```bash
cargo test --example debug --features btleplug-support parse_
```

Expected: compile failure because `parse_debug_mode_from`, `DebugCommand`, and `DebugMode` do not exist.

- [ ] **Step 3: Add CLI mode types after `SCAN_TIMEOUT`**

```rust
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
```

- [ ] **Step 4: Add usage text and parser after the mode types**

```rust
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
```

- [ ] **Step 5: Replace `main`**

```rust
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
```

- [ ] **Step 6: Replace discovery functions with reusable scan flow**

Replace `discover_and_select` and `find_ohea_locks` with:

```rust
async fn discover_and_select(adapter: &Adapter) -> Result<Peripheral> {
    println!("Scanning for {} devices ({:?})...", DEVICE_NAME, SCAN_TIMEOUT);

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

    Ok(devices.into_iter().nth(selection).expect("selection is in range"))
}

async fn discover_one_shot(adapter: &Adapter) -> Result<Peripheral> {
    println!("Scanning for {} devices ({:?})...", DEVICE_NAME, SCAN_TIMEOUT);

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
```

- [ ] **Step 7: Add one-shot executor before `run_interactive_session`**

```rust
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
```

- [ ] **Step 8: Verify this file**

Run:

```bash
cargo test --example debug --features btleplug-support
cargo run --example debug --features btleplug-support -- --help
```

Expected: tests pass; help prints usage and exits without Bluetooth initialization.

