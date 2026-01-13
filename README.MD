<div align="center">

## RunCat

</h1>

[![GitHub Stars](https://img.shields.io/github/stars/LuXv233/RunCat-rust?style=flat-square)](https://github.com/RayWangQvQ/BiliBiliToolPro/stargazers)
[![GitHub Forks](https://img.shields.io/github/forks/LuXv233/RunCat-rust?style=flat-square)](https://github.com/LuXv233/RunCat-rust/network)
[![GitHub Issues](https://img.shields.io/github/issues/LuXv233/RunCat-rust?style=flat-square)](https://github.com/LuXv233/RunCat-rust/issues)
[![GitHub Contributors](https://img.shields.io/github/contributors/LuXv233/RunCat-rust?style=flat-square)](https://github.com/LuXv233/RunCat-rust/graphs/contributors)
[![GitHub All Releases](https://img.shields.io/github/downloads/LuXv233/RunCat-rust/total?style=flat-square)](https://github.com/LuXv233/RunCat-rust/releases)
[![GitHub Release (latest SemVer)](https://img.shields.io/github/v/release/LuXv233/RunCat-rust?style=flat-square)](https://github.com/LuXv233/RunCat-rust/releases)
[![GitHub License](https://img.shields.io/github/license/LuXv233/RunCat-rust?style=flat-square)](https://github.com/LuXv233/RunCat-rust/blob/main/LICENSE)

</div>

  RunCat-rust是对原项目 RunCat365（[https://github.com/Kyome22/RunCat365](https://github.com/Kyome22/RunCat365)） 的 Rust 语言实现。

本项目以 Rust 重现原项目中“一只在系统托盘奔跑的小猫”的视觉效果，并添加其他程序逻辑，用于学习与个人使用。





## 重要说明 — 版权与侵权风险


- 美术资源（小猫动画/图像、图标等）来源于原项目 RunCat365，由原作者/原项目持有版权。代码作者在本仓库中保留了这些资源用于效果复现，但这些资源并非本仓库作者原创。
- 如果你是原资源的版权所有者并希望本仓库移除/修改致谢或资源，请通过 issue 或者邮件方式联系仓库维护者。





## 归属与协议


- 视觉资源、原始设计与灵感来自：Kyome22 / RunCat365 — https://github.com/Kyome22/RunCat365
- 本仓库代码遵循根目录 `LICENSE`（Apache-2.0）中声明的许可条款。





## 功能概览


- 在系统托盘显示一只小猫的奔跑动画。
- 奔跑速度会根据系统 CPU 占用比例动态变化（CPU 占用越高，Cat run得越快）。
- 添加“专注时间”功能，开启后在屏幕上显示当前时间。
- 添加深色/浅色切换功能。
- 添加开机自启功能:)。





## 构建与运行（Windows / bash）


开发环境要求：已安装 Rust（包含 cargo）。

在仓库根目录执行：


```bash
# 构建（Release）
cargo build --release

# 运行（Release 可执行文件路径）
.\target\release\run_cat.exe

# 或者直接用 cargo 运行（调试构建）
cargo run --release

```
注意：在 Windows 上直接运行可执行文件会在系统托盘创建图标，测试时请检查托盘区域。





## 常见问题


[Issues（议题）](https://github.com/LuXv233/RunCat-rust/issues)板块可以用来提交**Bug**和**建议**；

[Discussions（讨论）](https://github.com/LuXv233/RunCat-rust/discussions)板块可以用来**提问**和**讨论**。





## 版本发布及更新


当前正处于稳定的迭代开发中，想要有重要更新时收到通知的话，可以把仓库右上角的`Star`按钮点亮。
后续开发计划：

- 添加“专注时间”的编辑功能，实现时间、颜色、位置、大小的编辑功能。
- 增加在线更新功能。
- 小字:等我发现受启发的项目，就添加功能
- :)





## 致谢


感谢原作 Kyome22/RunCat365 的设计与美术资源（见上方链接），本项目受其启发。


非常感激/Very thankful












