# ⚡ xrayctl

<p align="center">
  <a href="README.md">🇬🇧 English</a> |
  <a href="README.ru.md">🇷🇺 <strong>Русский</strong></a>
</p>

<p align="center">
  <strong>Лёгкий CLI-менеджер для управления подписками и соединениями Xray.</strong>
</p>

<p align="center">
  <em>Управление подписками, выбор профилей, генерация конфигураций и запуск Xray с системной маршрутизацией через TUN.</em>
</p>

---

## ✨ Возможности

* 📡 Загрузка и обновление Xray-подписок
* 🔐 Поддержка запросов подписки с HWID
* 📋 Просмотр доступных профилей
* 🔎 Просмотр подробной информации о профилях
* 🎯 Выбор активного профиля
* ⚙️ Автоматическая генерация конфигурации Xray
* 🧪 Проверка конфигурации перед запуском
* 🌐 Системная маршрутизация через TUN
* 🔌 Локальный SOCKS5-прокси
* 🚀 Запуск и остановка Xray
* 📊 Проверка состояния Xray
* 📝 Сохранение логов Xray
* 🧩 Поддержка нескольких протоколов Xray

### 🔐 Протоколы

На данный момент протестированы:

* ⚡ Hysteria 2
* 🔒 VLESS + Reality + XHTTP

В дальнейшем список поддерживаемых протоколов может расширяться.

---

## 🛠️ Требования

`xrayctl` предназначен для Linux.

Необходимы:

* 🐧 Linux
* 🦀 Rust и Cargo
* ⚡ [Xray-core](https://github.com/XTLS/Xray-core)
* `sudo`
* `ip` из пакета `iproute2`

Проверить окружение:

```bash
rustc --version
cargo --version
xray version
ip -V
```

---

## 📦 Установка

### Сборка из исходников

Клонируйте репозиторий:

```bash
git clone https://github.com/lalolgl/xrayctl.git
cd xrayctl
```

Соберите release-версию:

```bash
cargo build --release
```

После сборки бинарный файл будет находиться здесь:

```text
target/release/xrayctl
```

Установите его локально:

```bash
mkdir -p ~/.local/bin
cp target/release/xrayctl ~/.local/bin/xrayctl
```

Проверьте установку:

```bash
xrayctl --help
```

> 💡 Убедитесь, что `~/.local/bin` добавлен в переменную `$PATH`.

---

## 🚀 Быстрый старт

### 1. Добавить подписку

```bash
xrayctl sub add "https://example.com/subscription"
```

URL подписки сохраняется в:

```text
~/.config/xrayctl/config.toml
```

### 2. Загрузить подписку

```bash
xrayctl sub fetch
```

Загруженная подписка сохраняется в:

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

Выбранный профиль сохраняется как активный.

### 5. Запустить Xray

```bash
xrayctl sub start
```

При запуске `xrayctl` автоматически:

1. 🔎 Загружает активный профиль
2. ⚙️ Подготавливает конфигурацию Xray
3. 🧪 Проверяет конфигурацию
4. 🚀 Запускает Xray
5. 🌐 Ожидает появления TUN-интерфейса
6. 🛣️ Настраивает необходимые маршруты
7. 🔌 Запускает локальный SOCKS5-прокси

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

## 📖 Список команд

| Команда                            | Описание                        |
| ---------------------------------- | ------------------------------- |
| `xrayctl sub add <URL>`            | Добавить подписку               |
| `xrayctl sub show`                 | Показать URL подписки           |
| `xrayctl sub fetch`                | Загрузить подписку              |
| `xrayctl sub update`               | Обновить подписку               |
| `xrayctl sub info`                 | Информация о подписке           |
| `xrayctl sub list`                 | Список доступных профилей       |
| `xrayctl sub debug`                | Отладка профилей                |
| `xrayctl sub show-profile <INDEX>` | Подробная информация о профиле  |
| `xrayctl sub use <INDEX>`          | Выбрать активный профиль        |
| `xrayctl sub generate`             | Сгенерировать конфигурацию Xray |
| `xrayctl sub start`                | Запустить Xray                  |
| `xrayctl sub stop`                 | Остановить Xray                 |
| `xrayctl sub status`               | Проверить состояние Xray        |

### Короткие опции

Также доступны сокращённые варианты:

```text
-s, --start
-x, --stop
-t, --status
-l, --list
-u, --update
-g, --generate
```

Например:

```bash
xrayctl sub -s
xrayctl sub -t
xrayctl sub -x
```

---

## 🌐 TUN-режим

`xrayctl` умеет настраивать Xray для работы через системный TUN-интерфейс.

Используется:

```text
Интерфейс: xray0
Адрес:      10.10.0.1/30
```

При запуске Xray `xrayctl`:

* создаёт необходимую маршрутизацию;
* сохраняет маршрут до удалённого Xray-сервера через исходный интерфейс;
* настраивает TUN-интерфейс Xray;
* заменяет системный маршрут по умолчанию маршрутом через TUN.

Это позволяет направлять трафик системы через Xray без необходимости отдельно настраивать каждое приложение.

---

## 🔌 SOCKS5

Xray также предоставляет локальный SOCKS5:

```text
127.0.0.1:9999
```

Проверить его работу можно командой:

```bash
curl --proxy socks5h://127.0.0.1:9999 https://example.com
```

---

## 📁 Конфигурация

Все рабочие файлы `xrayctl` находятся в:

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
| `xray.log`          | Вывод и ошибки Xray               |

---

## 🧪 Разработка

Клонируйте репозиторий:

```bash
git clone https://github.com/lalolgl/xrayctl.git
cd xrayctl
```

Запустить проект напрямую:

```bash
cargo run -- sub list
```

Собрать release:

```bash
cargo build --release
```

Запустить тесты:

```bash
cargo test
```

---

## 🗺️ Roadmap

### v0.1

* [x] Управление подписками
* [x] Просмотр профилей
* [x] Выбор профиля
* [x] Генерация конфигурации Xray
* [x] Проверка конфигурации
* [x] Hysteria 2
* [x] VLESS
* [x] TUN-маршрутизация
* [x] Запуск / остановка / статус
* [x] Базовый CLI-интерфейс

### Будущие версии

* [ ] Рефакторинг архитектуры проекта
* [ ] Улучшенная обработка ошибок
* [ ] Улучшенное управление жизненным циклом TUN
* [ ] Корректное завершение работы и очистка
* [ ] Автоматическое восстановление
* [ ] Диагностика соединения
* [ ] Управление DNS
* [ ] Поддержка IPv6
* [ ] Дополнительные протоколы Xray
* [ ] Автоматические интеграционные тесты
* [ ] Улучшенная проверка конфигураций
* [ ] Оптимизация производительности

> 💡 На текущем этапе основной приоритет — стабильность и хорошая основа проекта. Более глубокая оптимизация сетевой части и производительности может появиться в будущих версиях.

---

<p align="center">
  <sub>Built with 🦀 Rust and ❤️ for the Xray ecosystem.</sub>
</p>
