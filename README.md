<div align="center">

<img src="resource/app_icon.ico" alt="RunCat Logo" width="100">

# RunCat

**A cute running cat in your system tray, powered by Rust.**

English | [中文](README_zh.md)

[![GitHub Stars](https://img.shields.io/github/stars/LuXv233/RunCat-rust?style=for-the-badge)](https://github.com/LuXv233/RunCat-rust/stargazers)
[![GitHub Forks](https://img.shields.io/github/forks/LuXv233/RunCat-rust?style=for-the-badge)](https://github.com/LuXv233/RunCat-rust/network)
[![GitHub Issues](https://img.shields.io/github/issues/LuXv233/RunCat-rust?style=for-the-badge)](https://github.com/LuXv233/RunCat-rust/issues)
[![GitHub License](https://img.shields.io/github/license/LuXv233/RunCat-rust?style=for-the-badge)](https://github.com/LuXv233/RunCat-rust/blob/main/LICENSE)
[![GitHub Release](https://img.shields.io/github/v/release/LuXv233/RunCat-rust?style=for-the-badge)](https://github.com/LuXv233/RunCat-rust/releases)
[![Downloads](https://img.shields.io/github/downloads/LuXv233/RunCat-rust/total?style=for-the-badge)](https://github.com/LuXv233/RunCat-rust/releases)

</div>

---

RunCat-rust is a Rust reimplementation of [RunCat365](https://github.com/Kyome22/RunCat365), featuring an adorable running cat animation in the Windows system tray. The cat's speed dynamically reflects your CPU usage — the harder your machine works, the faster it runs.

## Demo

<div align="center">

| BubbleKitten | RunCat | Focus Time |
|:---:|:---:|:---:|
| ![BubbleKitten](images/气泡小猫演示.gif) | ![RunCat](images/奔跑小猫演示.gif) | ![Focus Time](images/时间演示.gif) |

</div>

## Features

<table>
<tr>
<td width="50%">

### System Tray Animation
A running cat lives in your system tray. Its speed scales with CPU usage in real time.

### Two Skins
- **BubbleKitten** — 10-frame cute bubble-style kitten
- **RunCat** — 5-frame classic running cat

### Color Modes
- Follow System (auto dark/light)
- Dark Mode
- Light Mode

</td>
<td width="50%">

### Focus Time
A floating clock with rainbow gradient text (8° hue rotation/sec), fully transparent and click-through.

### Edit Mode
Drag the time window anywhere on screen. Position is remembered across restarts.

### Auto-Start
Boot with Windows via Registry. One-click toggle from the tray menu.

### Persistent Settings
Theme, skin, time window visibility and position — all saved and restored automatically.

</td>
</tr>
</table>

## Build & Run

**Prerequisites:** [Rust](https://www.rust-lang.org/tools/install) (cargo)

```powershell
# Build release
cargo build --release

# Run
.\target\release\run_cat.exe

# Or run with cargo directly
cargo run --release
```

> The executable creates a system tray icon on launch. Check the tray area when testing.

## Usage

Right-click the cat icon in the system tray:

| Menu | Action |
|:---|:---|
| **Color Mode** | Switch between Follow System / Dark / Light |
| **Pet** | Switch between BubbleKitten / RunCat |
| **Show / Hide Time** | Toggle the floating time window |
| **Edit Mode** | Toggle drag mode for the time window |
| **Auto-Start** | Toggle boot with Windows |
| **Exit** | Quit the application |

## Copyright Notice

Art assets (cat animations, icons, etc.) are sourced from the original [RunCat365](https://github.com/Kyome22/RunCat365) project and are copyrighted by the original author. If you are the copyright holder and wish to request removal or modification, please open an issue or contact the maintainer.

## Acknowledgments

Thanks to [Kyome22 / RunCat365](https://github.com/Kyome22/RunCat365) for the original design and art assets. This project is inspired by their work.

## License

[Apache-2.0](LICENSE)
