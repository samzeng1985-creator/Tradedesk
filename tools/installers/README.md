# Windows 开发工具安装器

本目录中的安装程序仅用于本机开发环境，不提交到 Git 仓库。

## 1. Visual Studio Build Tools

运行 `vs_BuildTools.exe`，在安装器中选择：

- 使用 C++ 的桌面开发
- MSVC x64/x86 生成工具
- Windows 11 SDK

文件信息：

- 来源：Microsoft Visual Studio 官方下载页
- SHA-256：`746102400CD7B88C5AC2CCF66BA5FBCF7357710809BE0717CDA716BDEFF11817`
- Authenticode：有效，签名者为 Microsoft Corporation

## 2. Rust

运行 `rustup-init.exe`，输入 `1` 使用默认安装。安装完成后关闭并重新打开终端。

文件信息：

- 来源：Rust 官方 `static.rust-lang.org` x64 MSVC 分发地址
- SHA-256：`86478E53F769379D7F0EBFA7C9AA97CB76CA92233F79AA2CC0DBEE2EFAAC73C7`

## 3. 验证

回到项目根目录，在 PowerShell 执行：

```powershell
.\scripts\verify-dev.ps1
pnpm tauri dev
```
