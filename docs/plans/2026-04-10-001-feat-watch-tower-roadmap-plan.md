---
title: Watch Tower 桌面端迭代路线图
type: feat
status: active
date: 2026-04-11
revision: v0.3
origin: prd.md
---

# Watch Tower 桌面端迭代路线图（v0.3）

## Overview

本版路线图不再假设仓库仍处于“只有 Rust 启动骨架”的阶段，而是以当前代码库现实为起点重新排序后续工作：

- `v0.1` 基座已经完成，仓库已具备 `Tauri + React + TypeScript` 工程骨架、配置持久化、轮询/退避、共享模型和主窗口验证壳。
- 下一步最有杠杆的目标，不是先把主控台打磨成完整单窗产品，而是尽快交付桌面端真正区别于网页工具的“常驻价值”。
- 因此路线图改为：`最小主控台补强 -> 单 widget + tray 常驻 -> 提醒闭环 -> 高级桌面行为 -> 多窗口编排与发布收口`。

这份 v0.3 文档是对旧版路线图的重排，而不是全盘推翻：共享数据层、轮询宿主和主窗口验证壳仍然是后续所有版本的基础，不应重复建设。

## Revision Notes

相对旧版路线图，v0.3 的关键变化如下：

- 将 `v0.1` 明确标记为**已完成的基座阶段**，不再把它视为未来计划项。
- 将旧版“先做完整单窗版，再做常驻能力”的顺序调整为：
  - `v0.2` 只补足 widget/tray 依赖的最小主控台能力
  - `v0.3` 直接交付桌面常驻 MVP
- 将“提醒闭环”保留在 `v0.4`，避免在常驻体验尚未成立前就先制造提醒噪音。
- 将 `dock / auto-hide / hover / click-through` 继续后置到 `v0.5`，避免窗口系统复杂度过早拖慢主链路。

## Current State

当前仓库已经验证存在以下基础能力：

- 工程骨架：
  - `src-tauri/` 已建立 Tauri v2 桌面宿主结构
  - `src/` 已建立 React + Vite 前端入口
- 宿主能力：
  - `src-tauri/src/config/*` 已有配置仓储
  - `src-tauri/src/polling/*` 已有信号请求、调度与 backoff 骨架
  - `src-tauri/src/commands/mod.rs` 已提供前端与宿主的命令边界
- 共享模型：
  - `src/shared/config-model.ts`
  - `src/shared/alert-model.ts`
  - `src/shared/period-utils.ts`
  - `src/shared/view-models.ts`
- 可视验证壳：
  - `src/windows/main-dashboard/*` 已能承接 bootstrap、25 周期矩阵、60-bar 校验与 health/diagnostics 展示
- 计划上下文：
  - `docs/plans/2026-04-10-002-feat-watch-tower-v0-1-foundation-plan.md` 已标记为 `completed`

因此当前最重要的问题已经从“如何从 0 启动桌面项目”变成了“如何在现有基座上尽快形成真实桌面监控价值”。

## Problem Frame

从产品价值角度看，Watch Tower 的差异化不在于它有一个更复杂的配置页，而在于它能以桌面端形态常驻、扫一眼可读、异常时能及时唤醒用户。

这意味着后续路线图要避免两个常见偏差：

- 过早把大量精力投入到完整主控台 polish，结果先做出一个“更像后台管理页”的产品。
- 在常驻体验尚未成立时先上弹窗和系统通知，结果先制造噪音而不是价值。

所以这份路线图采用以下判断：

- **先补强主控台到“可运营”程度**，仅覆盖 widget/tray 依赖的最小配置与状态管理能力。
- **优先交付单 widget + tray 常驻闭环**，让产品先具备“不开主窗也有意义”的桌面价值。
- **提醒闭环放在常驻闭环之后**，确保通知是建立在稳定的常驻入口和单一状态源之上的。

## Requirements Trace

- R1. 支持用户配置 `API Key`、监控分组、周期范围、信号类型、轮询频率和基础窗口策略。
- R2. 支持桌面端常驻，至少包含主控台、edge widget、tray controller 三类入口。
- R3. 支持按组展示 25 个周期的信号总览，并可查看单级别最近 60 根 K 线映射。
- R4. 当发现 `read=false` 的新信号时，支持提醒闭环：检测、通知、跳转、标记已读、失败回滚。
- R5. 轮询机制具备最小间隔保护、`401`/`429`/`5xx` 显式状态、退避和 stale 反馈。
- R6. 一个 group 内只承载一个 `symbol`，多组由主控台和 widget 进行编排，不在单组视图中混 symbol。
- R7. 后续支持桌面高级行为：贴边、自动隐藏、观察态点击穿透、hover 唤醒、popup 堆叠复用。
- R8. 已完成的 `v0.1` 基座应被直接复用和延展，而不是在后续版本中重建另一套配置、轮询或共享模型。

## Scope Boundaries

- 本路线图只覆盖桌面端，不包含移动端开发。
- 不引入历史信号回溯；服务端当前仍只返回“最新警报”。
- 不引入 WebSocket；数据同步仍以轮询为主。
- 不以“首版就做到跨平台完全一致”为目标；Windows 优先，macOS 在抽象层预留兼容位。
- `03 Optional Market Overview Grid` 继续作为增强视图，不进入常驻 MVP 主链路。
- 后续版本若需要重构 `v0.1` 代码，必须以“复用与收敛”为目标，而不是平行造第二套运行时。

## Context & Verified Inputs

### Product Inputs

- `prd.md` 已明确：
  - 鉴权方式为 `x-api-key`
  - `UTC+0` 为大周期对齐基准
  - `symbol + period + signalType` 为已读组合键
  - 接口只返回最新警报，不返回历史序列
- `start.md` 已给出 25 个固定周期、轮询约束和基础桌面需求。

### Architecture & Design Inputs

- `docs/tauri-multi-window-architecture.md` 已定义：
  - `main dashboard`
  - `edge widget`
  - `alert popup`
  - `tray controller`
  四类窗口/入口职责
- Pencil 设计稿已覆盖：
  - `01 Bootstrap & Window Policy`
  - `02 Main Control Console`
  - `03 Optional Market Overview Grid`
  - `05 Slide-out Alert`
  - `06 Edge Widget`
  - `07 Dock & Auto-hide`
  - `08 Hover & Click-through`
  - `09 Multi-window Orchestration`

### Repo Reality

- `v0.1` 已完成的数据/宿主基座意味着：
  - 后续版本可以从“改造主控台与新增窗口”开始
  - 不必再把“创建 Tauri 工程、建立轮询器、沉淀 25 周期工具层”写成未来交付物
- 当前主窗口仍明显是“验证壳”，尚不足以作为长期使用的产品主控台：
  - 主要面向单组校验
  - 配置输入仍偏技术验证姿态
  - 尚未形成多组编辑、当前组选中、窗口策略等产品级交互

## Product Pressure Test

三条可选路径里，当前最值得押注的是“桌面常驻价值优先”：

- **完整主控台优先** 的问题：
  - 容易先做成一个更完整的配置/查看页，却仍没有桌面端辨识度。
- **提醒闭环优先** 的问题：
  - 在 widget/tray 还没建立前，通知会先成为噪音来源。
- **常驻价值优先** 的优势：
  - 更早验证 edge widget、tray、轮询健康状态这些桌面核心能力
  - 更快形成“不打开主控台也能使用”的产品价值
  - 为后续提醒闭环提供稳定的入口和状态基线

结论：下一阶段应该是“最小主控台补强 + 单 widget/tray 常驻 MVP”，而不是完整主控台 polish 或通知优先。

## Key Decisions

- 决策 1：将 `v0.1` 视为已完成的基座阶段，后续路线图从已有工程与验证壳继续推进。
  - 理由：仓库与完成状态文档都已证明这一步不是未来工作。

- 决策 2：`v0.2` 不追求把主控台做成最终形态，只补齐 `v0.3` 常驻 MVP 所依赖的最小产品能力。
  - 理由：主控台仍然重要，但它现在是“支撑常驻价值的准备面”，不是当前最高杠杆的终点。

- 决策 3：优先交付 `single edge widget + tray`，让产品先具备桌面端存在感。
  - 理由：这是 Watch Tower 区别于普通页面型工具的核心能力。

- 决策 4：提醒闭环放在常驻闭环之后。
  - 理由：先确保用户平时能扫一眼，再确保异常时能被稳定唤醒。

- 决策 5：高级桌面行为继续拆为独立阶段。
  - 理由：`auto-hide`、`hover wake`、`click-through` 都是高风险系统行为，应建立在已可用 widget 之上。

- 决策 6：共享 store、轮询状态和 diagnostics 继续维持单一真相来源。
  - 理由：主控台、widget、popup、tray 的一致性不能靠各自推断。

## High-Level Strategy

版本切分遵循三条原则：

- 每个版本都要形成一个可感知的用户价值闭环。
- 每个版本优先解决一类主问题，避免把配置、常驻、通知、复杂窗口行为混在一起。
- 只在现有基座上增量推进，不为“更干净的架构”牺牲产品节奏。

## Version Roadmap

| Version | Theme | Scope | Release bar |
|------|------|------|------|
| v0.1 | 基座与验证壳 | Tauri 工程、配置持久化、轮询、共享模型、25 周期与 60-bar 验证壳 | 已完成，作为后续所有版本的共享基础 |
| v0.2 | 主控台补强版 | 多组配置、当前组选中、窗口策略、布局预设、验证壳向产品主控台过渡 | 用户能在主控台完成常驻监控所需配置，而不是只做技术验证 |
| v0.3 | 桌面常驻 MVP | 单 edge widget、tray controller、左右停靠、主控台与 widget 联动、健康状态外显 | 不开主控台也能通过 widget + tray 监控当前组 |
| v0.4 | 提醒闭环版 | unread diff、单 symbol popup、系统通知、乐观已读回写、失败回滚 | 新信号能被发现、提醒、处理并同步 |
| v0.5 | 桌面行为增强版 | auto-hide、wake zone、hover 唤醒、观察态 click-through、widget 状态机 | widget 具备真正桌面组件的交互层级 |
| v0.6 | 编排与发布版 | 多 symbol popup 编排、可选总览 Grid、恢复面板、打包发布与试用文档 | 可作为首个对外试用版本分发 |

## Version Breakdown

### v0.1 基座与验证壳（已完成）

**目标:** 建立可运行桌面宿主、共享数据模型和可视验证壳，验证 25 周期、`UTC+0` 对齐和退避状态表达。

**当前结果:**
- 已具备 `Tauri + React + TypeScript` 工程骨架
- 已具备配置持久化、轮询、退避与 diagnostics 基础链路
- 已具备单组 25 周期矩阵与 60-bar 验证视图

**对后续的意义:**
- 后续所有版本都应在这条链路上演进，而不是重写

### v0.2 主控台补强版

**目标:** 把现有验证壳补到“足以支撑桌面常驻产品”的主控台程度，而不是追求完整 polish。

**包含范围:**
- 多 group 创建、编辑、删除与当前组选中
- 组级约束校验：一组只放一个 `symbol`
- API Key、轮询频率、窗口策略、默认 dock side 等配置持久化
- 主控台信息结构从验证导向调整为长期使用导向
- 列表/表格布局预设与基本密度设置
- 更清晰的错误反馈、空态和状态提示

**不包含:**
- edge widget
- tray
- 系统通知
- auto-hide / click-through

**版本验收:**
- 用户能从主控台完成长期使用所需的配置，而不需要改本地文件或依赖验证壳心智
- 主控台可以稳定切换和查看多个 group，但每个 group 仍保持单 symbol 约束
- `v0.3` 所需的 widget 配置项已经具备持久化来源

### v0.3 桌面常驻 MVP

**目标:** 让 Watch Tower 首次具备“不开主控台也有意义”的桌面价值。

**包含范围:**
- 单个 edge widget 窗口
- 左/right dock 基础能力
- tray controller
- 轮询状态、backoff 状态在 widget footer 与 tray 中可视化
- 主控台当前 group 与 widget 同步
- 恢复主控台、暂停轮询、退出应用等基础 tray 操作

**不包含:**
- unread popup
- 系统通知
- auto-hide
- click-through
- 多 widget 编排

**版本验收:**
- 用户关闭或隐藏主控台后，仍能通过 widget 和 tray 监控当前 group
- `running / paused / backoff / auth error` 至少在 widget 或 tray 中有明确反馈
- 常驻路径稳定，不要求依赖主控台保持打开

### v0.4 提醒闭环版

**目标:** 在已有常驻入口的基础上，把“发现新信号并处理”的闭环真正跑通。

**包含范围:**
- unread diff 检测
- 单 symbol slide-out popup
- 系统通知
- 标记已读
- 乐观更新与失败回滚
- 从 popup 跳转到主控台对应详情

**不包含:**
- 多 popup 编排
- auto-hide / click-through

**版本验收:**
- 新信号不会漏报
- 同一条未读不会被重复刷屏
- 用户处理已读后，状态能稳定回写；失败时会恢复并给出反馈

### v0.5 桌面行为增强版

**目标:** 把 widget 从“可用的窄窗”升级为真正贴边、可隐藏、可唤醒的桌面组件。

**包含范围:**
- auto-hide
- wake zone
- hover 唤醒
- 观察态 click-through
- `passive / hover / interactive` 状态机
- 新警报到来时的自动展开协同

**不包含:**
- 多 symbol popup 队列
- 市场总览 Grid

**版本验收:**
- widget 可以贴边隐藏并被 hover 唤醒
- 平台不支持的能力有可预期 fallback，而不是出现不可交互死态

### v0.6 编排与发布版

**目标:** 补齐多 symbol 提醒编排、增强视图和试用发布所需收尾工作。

**包含范围:**
- 多 symbol popup 复用与队列
- popup 可见上限与优先级管理
- 主控台未读队列 / 恢复面板
- 可选市场总览 Grid
- 安装包、README、默认权限与试用 checklist

**版本验收:**
- 多 symbol 同时告警不会互相遮挡或无限刷屏
- 应用可以交付给外部试用用户安装和体验

## Implementation Units

- [x] **Unit 1: 完成基座与验证壳**

**Goal:** 建立桌面宿主、共享模型、配置仓储、轮询/退避与主窗口验证壳。

**Status:** 已完成，对应 `docs/plans/2026-04-10-002-feat-watch-tower-v0-1-foundation-plan.md`

- [ ] **Unit 2: 将验证壳提升为可运营主控台**

**Goal:** 让当前主窗口从“单组验证工具”升级为后续 widget/tray 可依赖的产品主控台。

**Requirements:** R1, R3, R6, R8

**Dependencies:** Unit 1

**Likely files:**
- Modify: `src/windows/main-dashboard/index.tsx`
- Modify: `src/windows/main-dashboard/components/bootstrap-panel.tsx`
- Modify: `src/shared/config-model.ts`
- Modify: `src/shared/view-models.ts`
- Modify: `src/shared/events.ts`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/app_state.rs`
- Create: `src/windows/main-dashboard/components/group-list.tsx`
- Create: `src/windows/main-dashboard/components/group-editor.tsx`
- Create: `src/windows/main-dashboard/components/window-policy-form.tsx`
- Test: `src/shared/config-model.test.ts`
- Test: `src/windows/main-dashboard/components/group-editor.test.tsx`
- Test: `src/windows/main-dashboard/components/window-policy-form.test.tsx`

**Approach:**
- 把“Bootstrap & validation”心智改造成“持续配置与管理”心智。
- 先补齐 group 与窗口策略的核心数据模型，再升级主控台 UI，不做大规模视觉打磨。
- 将布局预设和当前组选中等能力收敛到主控台，而不是留给 widget 再自行决定。

**Test scenarios:**
- Happy path: 用户可以新增、编辑、删除 group，并持久化当前组选中状态。
- Edge case: 尝试在一个 group 中混入多个 symbol 时被明确阻止。
- Edge case: 保存窗口策略后，重启应用仍能恢复默认 dock side 和相关设置。
- Error path: API Key 或 API base URL 错误时，主控台保留可修复的错误态，而不是回到模糊空白。

**Verification:**
- `v0.3` 所需配置均可在主控台完成和恢复。
- 主控台已具备长期使用的最低产品门槛，但仍未进入过度 polish 阶段。

- [ ] **Unit 3: 交付单 widget + tray 常驻闭环**

**Goal:** 让应用首次具备桌面常驻能力，形成主控台之外的第一视线入口。

**Requirements:** R2, R3, R5, R6, R8

**Dependencies:** Unit 1, Unit 2

**Likely files:**
- Create: `src-tauri/src/windows/mod.rs`
- Create: `src-tauri/src/windows/edge_widget.rs`
- Create: `src-tauri/src/windows/positioning.rs`
- Create: `src-tauri/src/tray/mod.rs`
- Create: `src/windows/edge-widget/index.tsx`
- Create: `src/windows/edge-widget/components/period-row.tsx`
- Create: `src/windows/edge-widget/components/status-footer.tsx`
- Modify: `src/shared/events.ts`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/windows/positioning.rs`
- Test: `src-tauri/src/tray/mod.rs`
- Test: `src/windows/edge-widget/components/period-row.test.tsx`

**Approach:**
- 先只支持单个 widget 窗口和基础左右停靠。
- tray 先承接恢复主控台、暂停轮询、展示退避/异常状态三件关键事。
- widget footer 直接暴露轮询健康状态，确保桌面入口不是“只显示数据，不显示风险”。

**Test scenarios:**
- Happy path: 主控台关闭或隐藏后，widget 仍持续展示当前 group 的 25 周期状态。
- Happy path: tray 能恢复主控台、暂停轮询并反映当前运行态。
- Edge case: 当前 group 切换后，widget 同步刷新但仍保持单 symbol 视图。
- Error path: 429/5xx 进入 backoff 后，widget footer 和 tray 状态同步变化。

**Verification:**
- 用户不打开主控台也能完成日常扫一眼监控。
- 常驻链路在异常和暂停状态下仍具备可读性。

- [ ] **Unit 4: 打通 unread 提醒闭环**

**Goal:** 把新信号从“被轮询发现”推进到“被用户处理并回写”的完整流程。

**Requirements:** R4, R5, R8

**Dependencies:** Unit 1, Unit 2, Unit 3

**Likely files:**
- Create: `src/shared/unread-diff.ts`
- Create: `src-tauri/src/windows/alert_popup.rs`
- Create: `src-tauri/src/windows/queue.rs`
- Create: `src/windows/alert-popup/index.tsx`
- Create: `src/windows/alert-popup/components/alert-card.tsx`
- Modify: `src/shared/events.ts`
- Modify: `src-tauri/src/polling/mod.rs`
- Test: `src/shared/unread-diff.test.ts`
- Test: `src/windows/alert-popup/components/alert-card.test.tsx`
- Test: `src-tauri/src/windows/queue.rs`

**Approach:**
- unread diff 以“本轮快照 vs 上轮快照”为准，而不是基于 UI 自己缓存推断。
- 先做单 symbol popup，不提前解决多弹窗编排。
- 已读回写采用乐观更新 + 后台失败回滚。

**Test scenarios:**
- Happy path: 新的 `read=false` 信号到来时，能触发 popup 和系统通知。
- Edge case: 同一条未读重复轮询返回时，不重复制造新的提醒。
- Error path: 已读回写失败时，UI 恢复未读态并给出失败反馈。

**Verification:**
- 新信号从发现到处理的链路可演示、可恢复、不会重复刷屏。

- [ ] **Unit 5: 实现高级桌面行为状态机**

**Goal:** 让 widget 进入设计稿定义的桌面组件层级，而不只是一个固定窄窗。

**Requirements:** R2, R4, R7, R8

**Dependencies:** Unit 3, Unit 4

**Likely files:**
- Create: `src-tauri/src/windows/hover_state.rs`
- Create: `src-tauri/src/platform/mod.rs`
- Create: `src-tauri/src/platform/windows.rs`
- Create: `src-tauri/src/platform/macos.rs`
- Create: `src/shared/window-state.ts`
- Modify: `src-tauri/src/windows/edge_widget.rs`
- Modify: `src-tauri/src/windows/positioning.rs`
- Test: `src-tauri/src/windows/hover_state.rs`
- Test: `src-tauri/src/platform/windows.rs`
- Test: `src/shared/window-state.test.ts`

**Approach:**
- 把 auto-hide / hover / click-through 收敛为统一状态机，而不是散落在 UI 层判断。
- Windows 优先落地原生点击穿透；macOS 先提供抽象层和 fallback。
- 新警报到来时，widget 行为与提醒状态要联动。

**Test scenarios:**
- Happy path: widget 自动隐藏后可被 wake zone + hover 唤醒。
- Edge case: 鼠标短暂离开不会造成频繁闪烁或误隐藏。
- Error path: 平台不支持点击穿透时，回退到稳定可交互模式。

**Verification:**
- widget 具备贴边、隐藏、唤醒和观察态交互的完整闭环。

- [ ] **Unit 6: 完成多窗口编排与试用发布收口**

**Goal:** 在已有常驻和提醒能力上，解决多 symbol 并发提醒与试用交付问题。

**Requirements:** R2, R4, R5, R7, R8

**Dependencies:** Unit 4, Unit 5

**Likely files:**
- Modify: `src-tauri/src/windows/queue.rs`
- Modify: `src-tauri/src/windows/alert_popup.rs`
- Create: `src/windows/main-dashboard/components/unread-queue.tsx`
- Create: `src/windows/main-dashboard/components/recovery-panel.tsx`
- Create: `src/windows/market-overview/index.tsx`
- Create: `README.md`
- Test: `src/windows/main-dashboard/components/unread-queue.test.tsx`
- Test: `src/windows/market-overview/index.test.tsx`

**Approach:**
- 以 `symbol` 作为 popup 复用键；同 symbol 更新已有 popup，不重复开窗。
- visible popup 数量设上限，超出部分进入队列。
- 把 Grid 和恢复面板放在主链路稳定后再补齐。

**Test scenarios:**
- Happy path: 不同 symbol 短时间连续告警时，popup 按优先级进入可见栈或等待队列。
- Edge case: 屏幕高度不足时，低优先级 popup 进入队列而不是互相遮挡。
- Error path: backoff 状态期间不会无限制造普通级别 popup。

**Verification:**
- 多 symbol 告警管理可控、不重叠、不刷屏。
- 应用具备外部试用所需的打包与文档基础。

## System-Wide Impact

- **State truth:** 轮询结果、health、unread diff、popup queue、widget state 必须来自统一宿主状态，而不是窗口各自缓存。
- **Desktop differentiation:** `widget + tray + popup` 是这条产品线的核心，不应被主控台 polish 吞没节奏。
- **Failure visibility:** 鉴权失败、限流、服务错误不仅要在主控台可见，也要在 widget/tray 体现。
- **Platform risk:** click-through、dock、auto-hide 都有明显平台差异，必须在 `platform/*` 抽象层收口。
- **Carry cost control:** 当前阶段优先做一个 widget、一个 tray、一个 popup 流程，避免过早引入多窗口矩阵。

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| 把 `v0.2` 做成完整主控台项目，拖慢桌面常驻价值交付 | 明确 `v0.2` 只补强 widget/tray 所需最小能力 |
| 在常驻入口未成熟时先上通知，造成噪音和错报感 | 坚持 `widget/tray` 先于 popup/notification |
| 新增窗口后出现多处状态真相 | 继续以宿主共享状态为唯一数据来源 |
| 高级窗口行为过早进入主链路，导致桌面系统问题拖慢节奏 | `auto-hide/click-through` 独立为 `v0.5` |
| 因为已有 `v0.1` 验证壳而倾向于推倒重写 | 以“在现有主窗口和共享层上收敛演进”为默认策略 |

## Documentation / Operational Notes

- 进入 `v0.2` 前，应把主控台从“验证语气”改为“产品语气”，否则用户会继续把它当调试面板看待。
- 进入 `v0.3` 前，应准备一份 widget/tray 行为验收清单，覆盖隐藏主窗、恢复主窗、暂停轮询、backoff 状态外显。
- 进入 `v0.4` 前，应准备 unread diff 与已读回写的一致性检查清单，避免提醒链路一上来就重复刷屏。

## Sources & References

- Origin document: `prd.md`
- Supplemental requirements: `start.md`
- Related completed plan: `docs/plans/2026-04-10-002-feat-watch-tower-v0-1-foundation-plan.md`
- Architecture: `docs/tauri-multi-window-architecture.md`
- Design boards:
  - Pencil `01 Bootstrap & Window Policy`
  - Pencil `02 Main Control Console`
  - Pencil `03 Optional Market Overview Grid`
  - Pencil `05 Slide-out Alert`
  - Pencil `06 Edge Widget`
  - Pencil `07 Dock & Auto-hide`
  - Pencil `08 Hover & Click-through`
  - Pencil `09 Multi-window Orchestration`
