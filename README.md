# ⚡ xrayctl

<p align="center">
  <strong>🇬🇧 English</strong> &nbsp;|&nbsp; <a href="#-русский">🇷🇺 Русский</a>
</p>

**xrayctl** is a lightweight command-line manager for [Xray-core](https://github.com/XTLS/Xray-core).

It allows you to download Xray subscriptions, inspect available profiles, select a server, generate an Xray configuration and start Xray with a system TUN interface.

> 🚧 **Project status:** active development. The current version is functional and tested with real Xray configurations, but the project structure and CLI may change in future releases.

---

## ✨ Features

* 📡 Download Xray subscriptions
* 🔐 HWID-based subscription requests
* 📋 List available profiles
* 🔎 Inspect individual profiles
* 🎯 Select an active profile
* ⚙️ Automatically generate Xray configuration
* 🌐 System-wide TUN routing
* 🔌 Local SOCKS5 proxy at `127.0.0.1:9999`
* 🚀 Start and stop Xray
* 📊 Check Xray status
* 📝 Xray logging
* 🧩 Multiple Xray protocols

### 🔐 Supported protocols

Currently tested:

* ⚡ **Hysteria 2**
* 🔒 **VLESS + Reality + XHTTP**

More protocols can be added as the project evolves.

---

## 🛠️ Requirements

Before installing `xrayctl`, make sure your system has:

* 🐧 Linux
* 🦀 Rust + Cargo
* ⚡ Xray-core
* `sudo`
* `ip` from `iproute2`

Check Rust:

```bash
rustc --version
cargo --version
```

Check Xray:

```bash
xray version
```

Check `iproute2`:

```bash
ip -V
```

---

## 📥 Installation

### 1. Clone the repository

```bash
git clone https://github.com/lalolgl/xrayctl.git
cd xrayctl
```

### 2. Build the project

```bash
cargo build --release
```

The compiled binary will be located at:

```text
target/release/xrayctl
```

### 3. Install the binary

For a user-local installation:

```bash
mkdir -p ~/.local/bin
cp target/release/xrayctl ~/.local/bin/xrayctl
```

Make sure `~/.local/bin` is included in your `$PATH`.

Then verify the installation:

```bash
xrayctl --help
```

---

## 🚀 Quick Start

### 1. Add a subscription

```bash
xrayctl sub add "https://example.com/subscription"
```

The subscription configuration is stored in:

```text
~/.config/xrayctl/config.toml
```

### 2. Download the subscription

```bash
xrayctl sub fetch
```

Downloaded profiles are stored in:

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

The selected profile becomes the active configuration.

### 5. Start Xray

```bash
xrayctl sub start
```

`xrayctl` will:

1. 🔎 Load the active profile
2. ⚙️ Prepare the Xray configuration
3. 🧪 Validate the configuration
4. 🚀 Start Xray
5. 🌐 Wait for the TUN interface
6. 🛣️ Configure the required routes
7. 🔌 Expose SOCKS5 at `127.0.0.1:9999`

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

## 📖 Commands

| Command                            | Description                       |
| ---------------------------------- | --------------------------------- |
| `xrayctl sub add <URL>`            | Add a subscription                |
| `xrayctl sub show`                 | Show subscription URL             |
| `xrayctl sub fetch`                | Download subscription             |
| `xrayctl sub update`               | Update subscription               |
| `xrayctl sub info`                 | Show subscription information     |
| `xrayctl sub list`                 | List available profiles           |
| `xrayctl sub show-profile <INDEX>` | Show detailed profile information |
| `xrayctl sub use <INDEX>`          | Select active profile             |
| `xrayctl sub generate`             | Generate Xray configuration       |
| `xrayctl sub start`                | Start Xray                        |
| `xrayctl sub stop`                 | Stop Xray                         |
| `xrayctl sub status`               | Show Xray status                  |
| `xrayctl sub debug`                | Debug subscription profiles       |

---

## 📁 Configuration files

`xrayctl` stores its data in:

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
| `active.json`       | Currently selected profile   |
| `xray.json`         | Generated Xray configuration |
| `xray.pid`          | Xray process ID              |
| `xray.log`          | Xray logs                    |

---

## 🌐 TUN mode

When Xray is started, `xrayctl` creates and configures the Xray TUN interface:

```text
xray0
10.10.0.1/30
```

The default route is then directed through the Xray TUN interface.

The route to the remote Xray server is preserved through the original network interface to prevent the connection from routing back into the tunnel.

This allows the proxy to operate as a system-wide connection rather than requiring individual applications to support SOCKS5.

---

## 🔌 Local SOCKS5

Xray exposes a local SOCKS5 endpoint:

```text
127.0.0.1:9999
```

You can test it with:

```bash
curl --proxy socks5h://127.0.0.1:9999 https://example.com
```

---

## 🧪 Development

Clone the repository:

```bash
git clone https://github.com/lalolgl/xrayctl.git
cd xrayctl
```

Run the project directly:

```bash
cargo run -- sub list
```

Build a release version:

```bash
cargo build --release
```

Run the test suite:

```bash
cargo test
```

---

## 🗺️ Roadmap

### v0.1

* [x] Subscription management
* [x] Profile selection
* [x] Xray configuration generation
* [x] Hysteria 2
* [x] VLESS
* [x] TUN mode
* [x] Start / stop / status
* [x] Basic CLI UI

### v0.2+

* [ ] Refactor project architecture
* [ ] Improve error handling
* [ ] Better TUN management
* [ ] Graceful shutdown
* [ ] Automatic recovery
* [ ] IPv6 support
* [ ] DNS management
* [ ] Connection diagnostics
* [ ] More Xray protocols
* [ ] Automated tests
* [ ] Improved configuration validation

> 💡 Performance optimizations and deeper networking improvements are intentionally left for future releases. The current priority is stability, usability and a clean project foundation.

---

## 📄 License

License information will be added in a future release.

---

# 🇷🇺 Русский

**xrayctl** — лёгкий CLI-менеджер для управления подписками и конфигурациями [Xray-core](https://github.com/XTLS/Xray-core).

Программа позволяет загружать Xray-подписки, просматривать доступные профили, выбирать сервер, автоматически генерировать конфигурацию Xray и запускать соединение через системный TUN-интерфейс.

> 🚧 **Статус проекта:** активная разработка. Текущая версия уже работоспособна и протестирована на реальных конфигурациях Xray, однако структура проекта и CLI могут измениться в будущих версиях.

---

## ✨ Возможности

* 📡 Загрузка Xray-подписок
* 🔐 Запросы подписок с HWID
* 📋 Просмотр доступных профилей
* 🔎 Просмотр подробной информации о профиле
* 🎯 Выбор активного профиля
* ⚙️ Автоматическая генерация конфигурации Xray
* 🌐 Системная маршрутизация через TUN
* 🔌 Локальный SOCKS5 на `127.0.0.1:9999`
* 🚀 Запуск и остановка Xray
* 📊 Проверка состояния Xray
* 📝 Логирование Xray
* 🧩 Поддержка нескольких протоколов Xray

### 🔐 Поддерживаемые протоколы

На данный момент протестированы:

* ⚡ **Hysteria 2**
* 🔒 **VLESS + Reality + XHTTP**

В дальнейшем список поддерживаемых протоколов будет расширяться.

---

## 🛠️ Требования

Перед установкой `xrayctl` убедитесь, что в системе установлены:

* 🐧 Linux
* 🦀 Rust + Cargo
* ⚡ Xray-core
* `sudo`
* `ip` из пакета `iproute2`

Проверить Rust:

```bash
rustc --version
cargo --version
```

Проверить Xray:

```bash
xray version
```

Проверить `iproute2`:

```bash
ip -V
```

---

## 📥 Установка

### 1. Клонирование репозитория

```bash
git clone https://github.com/lalolgl/xrayctl.git
cd xrayctl
```

### 2. Сборка

```bash
cargo build --release
```

После сборки бинарник будет находиться здесь:

```text
target/release/xrayctl
```

### 3. Установка

Для установки только для текущего пользователя:

```bash
mkdir -p ~/.local/bin
cp target/release/xrayctl ~/.local/bin/xrayctl
```

Убедитесь, что `~/.local/bin` находится в `$PATH`.

После этого:

```bash
xrayctl --help
```

---

## 🚀 Быстрый старт

### 1. Добавить подписку

```bash
xrayctl sub add "https://example.com/subscription"
```

Конфигурация подписки будет сохранена в:

```text
~/.config/xrayctl/config.toml
```

### 2. Загрузить подписку

```bash
xrayctl sub fetch
```

Полученные профили сохраняются в:

```text
~/.config/xrayctl/subscription.json
```

### 3. Посмотреть профили

```bash
xrayctl sub list
```

Пример:

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

### 4. Выбрать профиль

```bash
xrayctl sub use 2
```

Выбранный профиль становится активным.

### 5. Запустить Xray

```bash
xrayctl sub start
```

`xrayctl` автоматически:

1. 🔎 Загружает активный профиль
2. ⚙️ Подготавливает конфигурацию Xray
3. 🧪 Проверяет её корректность
4. 🚀 Запускает Xray
5. 🌐 Ожидает появления TUN-интерфейса
6. 🛣️ Настраивает необходимые маршруты
7. 🔌 Открывает SOCKS5 на `127.0.0.1:9999`

### 6. Проверить состояние

```bash
xrayctl sub status
```

Пример:

```text
Xray
────────────────────────────────
● Running

  PID        12345
  SOCKS      127.0.0.1:9999
```

### 7. Остановить Xray

```bash
xrayctl sub stop
```

---

## 📖 Команды

| Команда                            | Описание                        |
| ---------------------------------- | ------------------------------- |
| `xrayctl sub add <URL>`            | Добавить подписку               |
| `xrayctl sub show`                 | Показать URL подписки           |
| `xrayctl sub fetch`                | Загрузить подписку              |
| `xrayctl sub update`               | Обновить подписку               |
| `xrayctl sub info`                 | Информация о подписке           |
| `xrayctl sub list`                 | Список профилей                 |
| `xrayctl sub show-profile <INDEX>` | Подробная информация о профиле  |
| `xrayctl sub use <INDEX>`          | Выбрать профиль                 |
| `xrayctl sub generate`             | Сгенерировать конфигурацию Xray |
| `xrayctl sub start`                | Запустить Xray                  |
| `xrayctl sub stop`                 | Остановить Xray                 |
| `xrayctl sub status`               | Проверить состояние Xray        |
| `xrayctl sub debug`                | Отладка профилей                |

---

## 📁 Файлы конфигурации

Все данные `xrayctl` хранятся в:

```text
~/.config/xrayctl/
├── config.toml
├── subscription.json
├── active.json
├── xray.json
├── xray.pid
└── xray.log
```

| Файл                | Назначение                        |
| ------------------- | --------------------------------- |
| `config.toml`       | Конфигурация подписки             |
| `subscription.json` | Загруженная подписка              |
| `active.json`       | Выбранный профиль                 |
| `xray.json`         | Сгенерированная конфигурация Xray |
| `xray.pid`          | PID процесса Xray                 |
| `xray.log`          | Логи Xray                         |

---

## 🌐 TUN-режим

При запуске Xray `xrayctl` создаёт и настраивает TUN-интерфейс:

```text
xray0
10.10.0.1/30
```

После этого маршрут по умолчанию направляется через Xray TUN.

При этом маршрут до удалённого Xray-сервера сохраняется через исходный сетевой интерфейс. Это необходимо, чтобы соединение с самим VPN-сервером не ушло обратно в туннель.

Таким образом, VPN работает на уровне всей системы, а приложениям не требуется самостоятельно поддерживать SOCKS5.

---

## 🔌 Локальный SOCKS5

Xray предоставляет локальный SOCKS5:

```text
127.0.0.1:9999
```

Проверить соединение можно через:

```bash
curl --proxy socks5h://127.0.0.1:9999 https://example.com
```

---

## 🧪 Разработка

Клонируйте репозиторий:

```bash
git clone https://github.com/lalolgl/xrayctl.git
cd xrayctl
```

Запустите проект напрямую:

```bash
cargo run -- sub list
```

Соберите release-версию:

```bash
cargo build --release
```

Запустите тесты:

```bash
cargo test
```

---

## 🗺️ Roadmap

### v0.1

* [x] Управление подписками
* [x] Выбор профилей
* [x] Генерация конфигурации Xray
* [x] Hysteria 2
* [x] VLESS
* [x] TUN-режим
* [x] Запуск / остановка / статус
* [x] Базовый CLI-интерфейс

### v0.2+

* [ ] Рефакторинг архитектуры проекта
* [ ] Улучшенная обработка ошибок
* [ ] Улучшенное управление TUN
* [ ] Корректное завершение работы
* [ ] Автоматическое восстановление
* [ ] Поддержка IPv6
* [ ] Управление DNS
* [ ] Диагностика соединения
* [ ] Дополнительные протоколы Xray
* [ ] Автоматические тесты
* [ ] Улучшенная проверка конфигураций

> 💡 Оптимизация производительности и более глубокая работа с сетевой частью специально оставлены на будущие версии. Сейчас основной приоритет — стабильность, удобство использования и чистая основа проекта.

---

## 📄 Лицензия

Информация о лицензии будет добавлена в одном из следующих релизов.

---

<p align="center">
  <a href="#-xrayctl">⬆️ Back to English</a>
</p>
