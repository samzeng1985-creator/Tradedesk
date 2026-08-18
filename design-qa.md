# Design QA — TradeDesk 0.6.1 业务单操作列

- source visual truth: user-provided business-order screenshot (kept outside the repository)
- implementation screenshot: `tmp/ui-qa/implementation.png`
- combined comparison: `tmp/ui-qa/comparison.png`
- viewport: 1904 × 1005 CSS px
- source dimensions/density: 1904 × 1005 px, 1×
- implementation dimensions/density: 1904 × 1005 px, 1×
- state: 2 条业务单，默认列表状态

## Full-view comparison

修复后的业务单中心保留现有信息架构、列宽、颜色、字号和操作顺序，只调整操作单元格的布局结构。

## Focused comparison

修复前，`<td>` 自身使用 `display: flex`，退出了原生表格布局，导致“编辑/归档”与对应业务单行错位。修复后，`<td>` 保持 `table-cell`，内部容器负责横向排列按钮。

## Required fidelity surfaces

- 两行操作区均与所属业务单行垂直居中。
- “编辑”仍在“归档”左侧，间距和颜色不变。
- 表头“操作”与操作内容保持同列。
- 搜索框输入 `0001` 后仅显示匹配行，清空后恢复 2 行。
- 浏览器控制台无 error 或 warn。

## Measurement and iteration history

1. 初始问题：操作列的表格单元格被设为 flex，破坏浏览器表格行布局。
2. 修复：将 `.row-actions` 移入 `<td>` 内部，并增加 `align-items: center`。
3. 复测：两个操作区的中心线相对所属行中心偏差均为 `0px`；操作单元格计算样式均为 `table-cell`。
4. 交互复测：搜索后可见行数为 1，清空后为 2；控制台无异常。

## Findings

没有剩余的 P0、P1 或 P2 视觉与交互问题。

final result: passed
