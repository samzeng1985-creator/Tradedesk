# TradeDesk Local

TradeDesk Local 是一款轻量、本地优先的外贸业务与单证桌面工具，同时面向 Windows 和 macOS。当前版本为单用户模式，业务数据默认存储在 SQLCipher 加密数据库中，不依赖本地 Web 服务或云端账号。

## 当前能力

- 产品、客户和供应商主数据：新建、编辑、搜索、停用及历史引用保护；客户档案包含三类地址、购买意向、客户分析、优劣势和主要联系人。
- 自选配置产品：组件库预录品名、型号/规格/材质、默认数量、单位、单价、品牌和备注；选择组件后生成可调数量与单价的组合报价清单，并冻结历史价格快照。
- 多语组件词库：类别、品名、品牌、规格、单位、备注、配置产品名称和配置说明自动记忆，可提前维护六种目标语言译文、停用词条并模糊搜索复用。
- 配置单输出：保存后的自选配置可导出英语、俄语、法语、西班牙语、葡萄牙语或阿拉伯语 A4 横向 PDF/CSV，并从最终 PDF 打印；中文业务词缺少目标语译文时禁止混合语言导出。
- 业务单：同一产品选择器支持标准产品和已保存的自选配置产品，冻结客户、产品、配置价格、币种、贸易术语、付款和交期快照。
- 采购与生产：按供应商拆单、采购成本、采购状态和六阶段生产进度。
- 销售单证：商业报价单、形式发票、外贸合同。
- 履约单证：商业发票、详细装箱单。
- 报价可直接转换为 PI/合同，PI 可转换为合同/商业发票，转换时复用已签发快照。
- 单证草稿、签发冻结、作废、新版本、历史搜索和基础一致性校验。
- Typst 专业 A4 模板，支持 PDF、CSV 和从最终 PDF 打印。
- PDF 导出记录路径、SHA-256、时间和对应的加密业务快照。

项目继续采用原生 React、CSS 与精简 Rust/SQL，未引入 UI 组件框架、Redux、ORM 或后台服务。

## 目录

```text
src/                    React 界面与前端领域类型
src-tauri/src/          Rust 核心、SQLCipher 存储与单证引擎
templates/base/         Typst 单证模板
docs/architecture/      轻量化架构约束
outputs/                PRD、研究记录与实施蓝图
scripts/                Windows/macOS 开发环境脚本
```

## 前端开发

需要 Node.js 20+ 和 pnpm：

```powershell
pnpm install
pnpm dev
```

构建验证：

```powershell
pnpm build
```

## 桌面开发环境

共同依赖：

- Rust stable（rustup）
- Node.js 20+
- pnpm
- Tauri CLI（项目开发依赖）
- Typst 0.15.1

Windows 还需要 Microsoft C++ Build Tools、WebView2 Runtime、Strawberry Perl 和 NASM。可在管理员 PowerShell 中执行：

```powershell
Set-ExecutionPolicy -Scope Process Bypass
.\scripts\setup-windows-dev.ps1
.\scripts\verify-dev.ps1
pnpm tauri dev
```

macOS 还需要 Xcode Command Line Tools：

```bash
bash scripts/setup-macos-dev.sh
pnpm tauri dev
```

生产构建：

```powershell
pnpm tauri build
```

## 单证使用路径

1. 创建加密工作区并录入公司名称。
2. 建立产品、组件、客户和供应商；需要组合报价时先保存自选配置产品，并在词库中补齐目标语言译文。
3. 新建业务单；从分组产品选择器中选择标准产品或币种一致的已完成自选配置，报价业务可从“报价”阶段开始。
4. 在“单证中心”创建商业报价单并签发。
5. 对已签发报价选择“转换单证”，生成 PI 或合同草稿。
6. 继续采购、生产和履约，生成发票与箱单。
7. PDF/CSV 默认保存到用户“文档/TradeDesk Exports”。

“打印”会先生成与预览一致的最终 PDF，再使用系统默认 PDF 阅读器打开。

## 数据与版本

- 数据库：SQLCipher Schema V9，客户档案、自选配置、多语组件词库和业务单产品来源通过增量迁移兼容旧数据，销售单证字段保存在向后兼容的 JSON 快照中。
- 旧单证在读取时自动补齐报价有效期和折扣默认值，无需手动迁移。
- 已签发版本只读；修改必须创建新版本。
- 当前 V1.0 仍为单用户。V1.1 再增加同机多人账号、角色权限和审核流。

远端仓库：<https://github.com/samzeng1985-creator/Tradedesk>
