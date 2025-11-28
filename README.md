# RunCat (Rust 重写)

本仓库是对原项目 RunCat365（[https://github.com/Kyome22/RunCat365](https://github.com/Kyome22/RunCat365)） 的 Rust 重写实现。
本项目仅保留原项目中“一只在系统托盘奔跑的小猫”的视觉效果，并以 Rust 重写程序逻辑，用于学习与个人使用。

## 重要说明 — 版权与侵权风险

- 美术资源（小猫动画/图像、图标等）来源于原项目 RunCat365，由原作者/原项目持有版权。代码作者在本仓库中保留了这些资源用于效果复现，但这些资源并非本仓库作者原创。

## 归属与致谢

- 视觉资源、原始设计与灵感来自：Kyome22 / RunCat365 — https://github.com/Kyome22/RunCat365
- 本仓库代码（Rust 重写部分）遵循根目录 `LICENSE`（Apache-2.0）中声明的许可条款。

如果你是原资源的版权所有者并希望本仓库移除/修改致谢或资源，请通过 issue 或者邮件方式联系仓库维护者（见下方“联系方式”）。

## 功能概览

- 在系统托盘显示一只小猫的奔跑动画。
- 奔跑速度会根据系统 CPU 占用比例动态变化（CPU 占用越高，小猫跑得越快）。
- 仅保留基础功能，便于在 Rust 中学习托盘应用与性能展示。

## 构建与运行（Windows / PowerShell）

开发环境要求：已安装 Rust（包含 cargo）。

在仓库根目录执行：

```powershell
# 构建（Release）
cargo build --release

# 运行（Release 可执行文件路径）
.\target\release\run_cat.exe

# 或者直接用 cargo 运行（调试构建）
cargo run --release


注意：在 Windows 上直接运行可执行文件会在系统托盘创建图标，测试时请检查托盘区域。


## 致谢

感谢原作 RunCat365 的设计与美术资源（见上方链接），本项目受其启发。

