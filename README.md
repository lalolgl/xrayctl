# ⚡ xrayctl

<p align="center">
  <a href="README.md">🇬🇧 <strong>English</strong></a> |
  <a href="README.ru.md">🇷🇺 Русский</a>
</p>

<p align="center">
  <strong>A lightweight CLI manager for Xray subscriptions and connections.</strong>
</p>

<p align="center">
  <em>Manage subscriptions, select profiles, generate configurations and run Xray with system-wide TUN routing.</em>
</p>

---

## ✨ Features

* 📡 Download and update Xray subscriptions
* 🔐 Support subscription requests with HWID
* 📋 List available profiles
* 🔎 Inspect individual profiles
* 🎯 Select an active profile
* ⚙️ Generate Xray configurations automatically
* 🧪 Validate configurations before starting Xray
* 🌐 System-wide TUN routing
* 🔌 Local SOCKS5 proxy
* 🚀 Start and stop Xray
* 📊 Monitor Xray status
* 📝 Store Xray logs
* 🧩 Support multiple Xray protocols

### 🔐 Tested protocols

Currently tested with:

* ⚡ Hysteria 2
* 🔒 VLESS + Reality + XHTTP

More protocols may be supported in the future.

---

## 🛠️ Requirements

`xrayctl` currently targets Linux systems.

You need:

* 🐧 Linux
* 🦀 Rust and Cargo
* ⚡ [Xray-core](https://github.com/XTLS/Xray-core)
* `sudo`
* `ip` from `iproute2`

Check your environment:

```bash
rustc --version
cargo --version
xray version
ip -V
```

---

## 📦 Installation

### From source

Clone the repository:

```bash
git clone https://github.com/lalolgl/xrayctl.git
cd xrayctl
```

Build the release version:

```bash
cargo build --release
```

The compiled binary will be available at:

```text
target/release/xrayctl
```

Install it locally:

```bash
mkdir -p ~/.local/bin
cp target/release/xrayctl ~/.local/bin/xrayctl
```

Then verify:

```bash
xrayctl --help
```

> 💡 Make sure `~/.local/bin` is included in your `$PATH`.

---

## 🚀 Quick Start

### 1. Add a subscription

```bash
xrayctl sub add "https://example.com/subscription"
```

The subscription URL is stored in:

```text
~/.config/xrayctl/config.toml
```

### 2. Download the subscription

```bash
xrayctl sub fetch
```

The downloaded subscription is saved to:

```text
~/.config/xrayctl/subscription.json
```

### 3. List profiles

```bash
xrayctl sub list
```

Example:

```text
Profiles
────────────────────────

Finland
────────────────────────
1. 🇫🇮 Finland
   Protocol: hysteria
   Group: Finland

2. 🇫🇮 Finland
   Protocol: vless
   Group: Finland

Germany
────────────────────────
3. 🇩🇪 Germany
   Protocol: vless
   Group: Germany

Total profiles: 3
```

### 4. Select a profile

```bash
xrayctl sub use 2
```

The selected profile is stored as the active profile.

### 5. Start Xray

```bash
xrayctl sub start
```

When starting Xray, `xrayctl` automatically:

1. 🔎 Loads the active profile
2. ⚙️ Prepares the Xray configuration
3. 🧪 Validates the configuration
4. 🚀 Starts Xray
5. 🌐 Waits for the TUN interface
6. 🛣️ Configures the required routes
7. 🔌 Provides a local SOCKS5 proxy

### 6. Check status

```bash
xrayctl sub status
```

Example:

```text
Xray
────────────────────────────────
● Running

  PID        12345
  SOCKS      127.0.0.1:9999
```

### 7. Stop Xray

```bash
xrayctl sub stop
```

---

## 📖 Command Reference

| Command                            | Description                          |
| ---------------------------------- | ------------------------------------ |
| `xrayctl sub add <URL>`            | Add a subscription                   |
| `xrayctl sub show`                 | Show the configured subscription URL |
| `xrayctl sub fetch`                | Download the subscription            |
| `xrayctl sub update`               | Update the subscription              |
| `xrayctl sub info`                 | Show subscription information        |
| `xrayctl sub list`                 | List available profiles              |
| `xrayctl sub debug`                | Debug subscription profiles          |
| `xrayctl sub show-profile <INDEX>` | Show detailed profile information    |
| `xrayctl sub use <INDEX>`          | Select an active profile             |
| `xrayctl sub generate`             | Generate an Xray configuration       |
| `xrayctl sub start`                | Start Xray                           |
| `xrayctl sub stop`                 | Stop Xray                            |
| `xrayctl sub status`               | Show Xray status                     |

### Short options

The following shortcuts are also available:

```text
-s, --start
-x, --stop
-t, --status
-l, --list
-u, --update
-g, --generate
```

For example:

```bash
xrayctl sub -s
xrayctl sub -t
xrayctl sub -x
```

---

## 🌐 TUN Mode

`xrayctl` can configure Xray to operate through a system-wide TUN interface.

The generated configuration uses:

```text
Interface: xray0
Address:   10.10.0.1/30
```

When Xray starts, `xrayctl`:

* creates the required routing configuration;
* preserves the route to the remote Xray server through the original uplink;
* configures the Xray TUN interface;
* replaces the system default route with the TUN route.

This allows supported traffic to be routed through Xray without configuring every application individually.

---

## 🔌 SOCKS5

Xray also provides a local SOCKS5 endpoint:

```text
127.0.0.1:9999
```

You can test it with:

```bash
curl --proxy socks5h://127.0.0.1:9999 https://example.com
```

---

## 📁 Configuration

`xrayctl` stores its runtime data in:

```text
~/.config/xrayctl/
├── config.toml
├── subscription.json
├── active.json
├── xray.json
├── xray.pid
└── xray.log
```

| File                | Purpose                      |
| ------------------- | ---------------------------- |
| `config.toml`       | Subscription configuration   |
| `subscription.json` | Downloaded subscription      |
| `active.json`       | Selected profile             |
| `xray.json`         | Generated Xray configuration |
| `xray.pid`          | Running Xray process ID      |
| `xray.log`          | Xray output and errors       |

---

## 🧪 Development

Clone the repository:

```bash
git clone https://github.com/lalolgl/xrayctl.git
cd xrayctl
```

Run directly with Cargo:

```bash
cargo run -- sub list
```

Build a release:

```bash
cargo build --release
```

Run tests:

```bash
cargo test
```

---

## 🗺️ Roadmap

### v0.1

* [x] Subscription management
* [x] Profile listing
* [x] Profile selection
* [x] Xray configuration generation
* [x] Configuration validation
* [x] Hysteria 2 support
* [x] VLESS support
* [x] TUN routing
* [x] Start / stop / status
* [x] Basic CLI interface

### Future releases

* [ ] Refactor the project architecture
* [ ] Improve error handling
* [ ] Improve TUN lifecycle management
* [ ] Graceful shutdown and cleanup
* [ ] Automatic recovery
* [ ] Connection diagnostics
* [ ] DNS management
* [ ] IPv6 support
* [ ] Additional Xray protocols
* [ ] Automated integration tests
* [ ] Improved configuration validation
* [ ] Performance optimizations

> 💡 The current priority is stability and a clean project foundation. More advanced networking and performance improvements can be introduced in future versions.

---

<p align="center">
  <sub>Built with 🦀 Rust and ❤️ for the Xray ecosystem.</sub>
</p>
