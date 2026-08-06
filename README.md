# TradeDesk Local

轻量、本地优先的外贸业务与单证桌面工具。首版范围锁定为单用户、Windows/macOS、产品/客户/供应商主数据、采购与生产里程碑、业务单和单证快照。

## 当前进度

- 已完成加密工作区创建、密码解锁和手动锁定。
- 产品、客户和供应商支持真实本地新建、编辑、搜索、停用与重启持久化。
- 业务单中心已完成：客户、订单产品、商业条款、金额与业务快照。
- 采购与生产中心已接入真实加密数据：业务单拆分采购、供应商分配、采购状态和六个生产里程碑。
- 工作台已显示真实销售金额、采购成本、生产进度、可发货数量和异常节点。
- 单证中心已完成真实纵向切片：商业发票、详细装箱单、外贸合同的创建、编辑、预览、签发、作废和新版本。
- 已建立 Tauri 2 + Rust 桌面壳。
- 已建立 SQLCipher 默认加密数据库、增量迁移和基础审计事件。
- 已使用 Typst 0.15.1 建立三套专业 A4 模板，支持 PDF、CSV 和从最终 PDF 打印。
- 前端生产构建约 252 KB JavaScript，gzip 约 76 KB；未引入 UI 框架、Redux 或 ORM。

## 目录

```text
src/                    React 界面与前端领域类型
src-tauri/src/          Rust 核心、SQLCipher 存储、单证快照
templates/base/         Typst 单证模板
docs/architecture/      轻量化架构约束
outputs/                PRD 与实施蓝图
```

## 前端开发

需要 Node.js 20+ 和 pnpm。

```powershell
pnpm install
pnpm dev
```

构建验证：

```powershell
pnpm build
```

## 桌面端开发环境

共同环境：

- Rust stable（rustup 安装）
- Node.js 20+
- pnpm
- Tauri CLI（已经作为项目开发依赖）

Windows 还需要：

- Microsoft C++ Build Tools，选择“使用 C++ 的桌面开发”
- WebView2 Runtime
- Strawberry Perl（仅用于编译静态 OpenSSL/SQLCipher）
- NASM（仅用于加速 Windows 下的 OpenSSL 构建）
- Typst 0.15.1（安装脚本下载到项目 `tools/typst`，发布时与主程序一起分发）

可使用 WinGet 安装两个构建工具：

```powershell
winget install --exact --id StrawberryPerl.StrawberryPerl --source winget
winget install --exact --id NASM.NASM --source winget
```

Windows 可在管理员 PowerShell 中执行最小化安装脚本：

```powershell
Set-ExecutionPolicy -Scope Process Bypass
.\scripts\setup-windows-dev.ps1
.\scripts\verify-dev.ps1
```

macOS 还需要：

- Xcode Command Line Tools
- 执行 `bash scripts/setup-macos-dev.sh` 下载对应 Apple Silicon/Intel 的 Typst 0.15.1 渲染器

环境就绪后运行：

```powershell
pnpm tauri dev
```

生产构建必须启用 Tauri 自定义协议；项目已将其设为默认 Cargo 特性。完整构建仍推荐：

```powershell
pnpm tauri build
```

本机已验证 WebView2、Rust、Microsoft C++ Build Tools、SQLCipher 与 Tauri 生产构建。

## 单证导出

- PDF 与 CSV 默认保存到用户“文档/TradeDesk Exports”。
- 每次 PDF 导出会保存 SHA-256、导出时间和对应的加密业务快照。
- “打印”先生成与预览一致的 PDF，再调用系统默认 PDF 阅读器进行打印。
- Typst 是独立渲染器，不需要最终用户安装 Node.js、Python 或办公软件。

## 版本仓库

项目远端仓库为：<https://github.com/samzeng1985-creator/Tradedesk>

`main` 分支上的 GitHub Actions 会验证前端构建，并分别在 Windows 和 macOS 上执行 Rust 原生代码检查。

## 轻量化守则

- 页面状态优先使用 React 原生状态；达到跨模块共享阈值后再引入状态方案。
- 数据访问只保留少量显式 SQL，不引入 ORM。
- 每类单证共享一个快照模型，模板只负责版式差异。
- 首版不加入多人协作、云同步、Excel 导出或复杂排产。
