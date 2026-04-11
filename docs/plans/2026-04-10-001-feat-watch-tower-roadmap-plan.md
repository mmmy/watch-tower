---
title: Watch Tower 桌面端迭代路线图
type: feat
status: active
date: 2026-04-11
revision: v0.4
origin: prd.md
---

# Watch Tower 桌面端迭代路线图（v0.4）

## Overview

本版路线图以当前仓库现实为起点重新收口后续节奏：

- `v0.1` 基座已经完成，`Tauri + React + TypeScript` 工程、配置持久化、轮询/退避、共享模型和主窗口验证壳已就位。
- `v0.2` 主控台补强也已经完成，仓库已具备多 group 管理、当前组选中、布局预设和基础窗口策略。
- 因此当前最高杠杆目标，不再是继续打磨主控台，而是尽快把桌面端做成一个“每天真会开着用”的常驻产品。

基于这一步判断，后续路线图调整为：

`已完成 v0.1/v0.2 -> resident daily-driver MVP -> 提醒闭环 -> 高级桌面行为 -> 多窗口编排与发布收口`

这份 `v0.4` 文档不是推翻旧路线，而是把已完成工作正式沉淀为现状，并把 `v0.3` 收窄为一个更真实、更可验证的桌面常驻切片。

## Revision Notes

相对 `v0.3` 版路线图，本次修订的关键变化如下：

- 将 `v0.2` 从“下一步主控台补强”更新为**已完成阶段**，因为仓库和完成计划都已证明这一步已经落地。
- 将 `v0.3` 明确重定义为**resident daily-driver MVP**：
  - 优先验证 `widget + tray + resident runtime`
  - 目标是“不开主控台也能扫一眼使用”，而不是“提醒完整”或“多窗口架构完整”
- 将提醒边界重新切清：
  - `v0.3` 不做 popup / 系统通知
  - `v0.3` 只承接常驻扫一眼价值和健康状态外显
  - 提醒闭环继续留在 `v0.4`
- 明确 `v0.3` 不支持在 widget/tray 中切换 group：
  - 常驻面只显示当前主控台选中的 group
  - group 切换继续留在主控台完成
- 保持 `auto-hide / hover wake / click-through` 在 `v0.5`，避免高级桌面行为侵占 `v0.3` 主链路

## Current State

当前仓库已经验证存在以下基础能力：

- 工程与宿主基础：
  - `src-tauri/` 已建立 Tauri v2 宿主结构
  - `src/` 已建立 React + Vite 前端入口
  - `src-tauri/src/config/*` 已有配置仓储
  - `src-tauri/src/polling/*` 已有信号请求、调度与 backoff 骨架
  - `src-tauri/src/commands/mod.rs` 已提供前端与宿主命令边界
- 共享模型：
  - `src/shared/config-model.ts`
  - `src/shared/alert-model.ts`
  - `src/shared/period-utils.ts`
  - `src/shared/view-models.ts`
  - `src/shared/events.ts`
- 主控台能力：
  - `src/windows/main-dashboard/*` 已从验证壳演进为可长期使用的主控台
  - 已具备 group list / group editor / window policy / layout preset / health / diagnostics 等面板
- 计划上下文：
  - `docs/plans/2026-04-10-002-feat-watch-tower-v0-1-foundation-plan.md` 已标记为 `completed`
  - `docs/plans/2026-04-11-003-feat-watch-tower-v0-2-main-dashboard-plan.md` 已标记为 `completed`

因此当前最重要的问题已经不再是“如何把桌面项目搭起来”或“如何让主控台能管理配置”，而是：

**当主控台隐藏后，Watch Tower 是否已经具备值得常驻桌面的独立价值。**

## Problem Frame

Watch Tower 的桌面差异化，不在于它又多一个配置页，而在于它可以持续存在于桌面边缘，被用户低成本扫一眼，并在异常时有进一步演进的空间。

从当前阶段看，真正需要先验证的是：

- 用户不打开主控台时，是否仍能通过常驻入口持续获得监控价值。
- 常驻入口是否能稳定表达 `running / paused / backoff / auth error / stale` 这些运行语义。
- 主控台已经沉淀出的 `selectedGroupId`、窗口策略和共享状态，是否足以直接驱动 resident runtime，而不需要再造第二套桌面配置逻辑。

这意味着后续路线图要继续避免三个偏差：

- 把 `v0.3` 做成“桌面多窗口架构演示”，却还没有形成个人 daily-driver 价值。
- 把提醒闭环提前塞进 `v0.3`，导致常驻 MVP 同时背负通知噪音和状态一致性风险。
- 为了减少主控台依赖，过早把 group 切换塞进 tray/widget，导致 resident MVP scope 膨胀。

所以这份路线图采用以下判断：

- **先把 resident loop 做成立**，验证 `widget + tray + health` 本身就有存在价值。
- **提醒闭环后置到 v0.4**，建立在稳定常驻入口之上。
- **group 切换继续留在主控台**，避免把 resident MVP 做成第二套迷你控制台。

## Requirements Trace

- R1. 支持用户配置 `API Key`、监控分组、周期范围、信号类型、轮询频率和基础窗口策略。
- R2. 支持桌面端常驻，至少包含主控台、edge widget、tray controller 三类入口。
- R3. 支持按组展示 25 个周期的信号总览，并可查看单级别最近 60 根 K 线映射。
- R4. 当发现 `read=false` 的新信号时，支持提醒闭环：检测、通知、跳转、标记已读、失败回滚。
- R5. 轮询机制具备最小间隔保护、`401`/`429`/`5xx` 显式状态、退避和 stale 反馈。
- R6. 一个 group 内只承载一个 `symbol`，多组由主控台进行管理，resident surface 只展示当前 selected group。
- R7. 后续支持桌面高级行为：贴边、自动隐藏、观察态点击穿透、hover 唤醒、popup 堆叠复用。
- R8. 已完成的 `v0.1` 基座和 `v0.2` 主控台产物应被直接复用和延展，而不是在后续版本中重建另一套配置、轮询或状态层。
- R9. `v0.3` 常驻 MVP 不承担提醒闭环和 group switching；它只负责把当前选中 group 变成稳定可读的桌面入口。

## Scope Boundaries

- 本路线图只覆盖桌面端，不包含移动端开发。
- 不引入历史信号回溯；服务端当前仍只返回“最新警报”。
- 不引入 WebSocket；数据同步仍以轮询为主。
- 不以“首版就做到跨平台完全一致”为目标；Windows 优先，macOS 在抽象层预留兼容位。
- `03 Optional Market Overview Grid` 继续作为增强视图，不进入 resident MVP 主链路。
- `v0.3` 不实现 popup、系统通知、已读回写或 unread queue。
- `v0.3` 不实现 `auto-hide / hover wake / click-through`。
- `v0.3` 不在 tray/widget 中提供 group 切换；切换继续留在主控台。
- 后续版本若需要重构现有代码，必须以“复用与收敛”为目标，而不是平行造第二套运行时。

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
  - 后续版本可以直接站在共享配置、轮询状态和 diagnostics 之上推进
  - 不必再把“创建 Tauri 工程、建立轮询器、沉淀 25 周期工具层”写成未来交付物
- `v0.2` 已完成的主控台能力意味着：
  - 多 group 编辑、当前组选中、布局预设和基础窗口策略已经有持久化来源
  - resident runtime 可以直接消费主控台已有配置，而不是再发明一套“widget 专用配置流程”
- 当前仍未完成的关键差距是：
  - 主窗口隐藏后，缺少真正长期存在的桌面入口
  - tray 还没有成为桌面会话控制器
  - 当前产品价值仍过度依赖主控台保持打开

## Product Pressure Test

围绕“下一步到底先做什么”，当前存在三条可选路径：

- **架构完整优先**
  - 优点：多窗口 runtime 边界会更整齐。
  - 问题：容易先得到一个技术里程碑，而不是一个每天真会打开的产品切片。
- **外部演示优先**
  - 优点：更快形成可展示故事。
  - 问题：容易为了 demo 一次性塞入提醒和编排，导致 `v0.3` scope 失真。
- **daily-driver 常驻优先**
  - 优点：
    - 更快验证桌面端真正差异化的价值
    - 更快暴露 resident runtime 的真实问题
    - 能直接检验当前主控台产物是否已足够支撑常驻入口
  - 代价：
    - 必须严格控制 `v0.3` scope，拒绝把提醒和高级行为提前混入

结论：下一阶段应该明确押注 **daily-driver 常驻优先**。

同时，围绕提醒边界与 group 切换边界，也需要继续保持克制：

- `v0.3` 采用 **纯常驻扫一眼版**，不加入 popup / 系统通知。
- `v0.3` 采用 **当前组选中复用策略**，不在 tray/widget 中重做 group switching。

## Key Decisions

- 决策 1：将 `v0.1` 和 `v0.2` 视为已完成阶段，后续路线图从已有主控台与共享状态继续推进。
  - 理由：仓库与完成状态文档都已证明这两步不是未来工作。

- 决策 2：`v0.3` 以 **resident daily-driver MVP** 为目标，而不是“架构完整”或“提醒完整”。
  - 理由：当前最值得验证的是桌面端的常驻价值是否成立。

- 决策 3：`v0.3` 只交付 `widget + tray + resident runtime health`，不带 popup / 系统通知。
  - 理由：先建立稳定、低噪音的扫一眼体验，再进入提醒闭环。

- 决策 4：`v0.3` 的 resident surface 只显示当前主控台 selected group，不承担 group switching。
  - 理由：减少 resident MVP scope，避免 tray/widget 演变成第二套迷你控制台。

- 决策 5：提醒闭环保留在 `v0.4`。
  - 理由：提醒应建立在稳定常驻入口和统一状态源之上，而不是与 resident MVP 同时冒险。

- 决策 6：高级桌面行为继续拆到 `v0.5`。
  - 理由：`auto-hide`、`hover wake`、`click-through` 都是高风险系统行为，应在 widget 已可稳定日常使用后再做。

- 决策 7：共享 store、轮询状态和 diagnostics 继续维持单一真相来源。
  - 理由：主控台、widget、tray、popup 的一致性不能靠各自推断。

## High-Level Strategy

版本切分遵循四条原则：

- 每个版本都要形成一个可感知的用户价值闭环。
- `v0.3` 先验证“主窗关闭后仍有意义”，再考虑“异常时如何打扰用户”。
- resident surface 优先做“当前组选中视图”，不把 group orchestration 提前摊进 tray/widget。
- 只在现有基座和主控台上增量推进，不为了“更干净的未来架构”牺牲当前产品节奏。

## Version Roadmap

| Version | Theme | Scope | Release bar |
|------|------|------|------|
| v0.1 | 基座与验证壳 | Tauri 工程、配置持久化、轮询、共享模型、25 周期与 60-bar 验证壳 | 已完成，作为后续所有版本的共享基础 |
| v0.2 | 主控台补强版 | 多组配置、当前组选中、窗口策略、布局预设、主控台产品化 | 已完成，作为 resident runtime 的配置与状态来源 |
| v0.3 | Resident Daily-Driver MVP | 单 edge widget、tray controller、左右停靠、主窗隐藏后继续常驻、health/backoff/auth 状态外显 | 不开主控台也能通过 widget + tray 持续监控当前 group |
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

### v0.2 主控台补强版（已完成）

**目标:** 把验证壳升级为可长期使用的主控台，为 resident runtime 提供稳定配置与当前组选中状态。

**当前结果:**
- 已具备多 group 创建、编辑、删除与当前组选中
- 已具备基础窗口策略、布局预设和主控台信息重组
- 已具备 resident runtime 后续可直接消费的 `selectedGroupId` 和配置快照

### v0.3 Resident Daily-Driver MVP

**目标:** 让 Watch Tower 首次具备“主窗关闭后仍值得常驻桌面”的日常使用价值。

**包含范围:**
- 单个 edge widget 窗口
- 左/right dock 基础能力
- tray controller
- 主窗隐藏/恢复与 resident runtime 生命周期打通
- 轮询状态、backoff 状态、鉴权失败、stale 状态在 widget footer 与 tray 中可视化
- 主控台当前 selected group 与 widget 同步
- 恢复主控台、暂停轮询、恢复轮询、退出应用等基础 tray 操作

**不包含:**
- unread popup
- 系统通知
- 已读回写
- auto-hide
- click-through
- hover wake
- 多 widget 编排
- 不打开主控台就切换 group

**版本验收:**
- 用户关闭或隐藏主控台后，仍能通过 widget 和 tray 监控当前 group 的 25 周期状态
- `running / paused / backoff / auth error / stale` 至少在 widget 或 tray 中有明确反馈
- 常驻链路稳定，不要求主控台保持打开
- 用户若要切换 group，仍回到主控台操作，但这不会破坏 resident loop 的日常价值

### v0.4 提醒闭环版

**目标:** 在已有 resident loop 的基础上，把“发现新信号并处理”的闭环真正跑通。

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

**目标:** 把 widget 从“可用的常驻窄窗”升级为真正贴边、可隐藏、可唤醒的桌面组件。

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

- [x] **Unit 2: 将验证壳提升为可运营主控台**

**Goal:** 让主窗口从“单组验证工具”升级为 resident runtime 可依赖的产品主控台。

**Status:** 已完成，对应 `docs/plans/2026-04-11-003-feat-watch-tower-v0-2-main-dashboard-plan.md`

- [ ] **Unit 3: 交付 resident daily-driver MVP**

**Goal:** 让应用首次具备主控台之外的长期常驻入口，并验证“当前组选中 + widget + tray”已足够支撑日常桌面使用。

**Requirements:** R2, R3, R5, R6, R8, R9

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
- Modify: `src-tauri/src/app_state.rs`
- Modify: `src/windows/main-dashboard/components/window-policy-form.tsx`
- Test: `src-tauri/src/windows/positioning.rs`
- Test: `src-tauri/src/tray/mod.rs`
- Test: `src/windows/edge-widget/components/period-row.test.tsx`

**Approach:**
- 先只支持单个 widget 窗口和基础左右停靠。
- resident surface 直接复用主控台现有 `selectedGroupId`，不在 tray/widget 重建 group 管理交互。
- tray 先承接恢复主控台、暂停/恢复轮询、退出应用和运行状态外显四件关键事。
- widget footer 直接暴露轮询健康状态，确保桌面入口不是“只显示数据，不显示风险”。
- 主窗隐藏后 resident runtime 仍然存活，避免“名义上常驻，实际上仍依赖主窗保持打开”。

**Test scenarios:**
- Happy path: 主控台关闭或隐藏后，widget 仍持续展示当前 selected group 的 25 周期状态。
- Happy path: tray 能恢复主控台、暂停轮询、恢复轮询并反映当前运行态。
- Edge case: 当前 group 在主控台切换后，widget 同步刷新但 tray/widget 不承担切换入口。
- Error path: `401`、`429`、`5xx` 或 stale 状态下，widget footer 和 tray 状态同步变化。

**Verification:**
- 用户不打开主控台也能完成日常扫一眼监控。
- resident surface 已具备“个人 daily-driver”价值，但仍保持低噪音和低 scope。

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
- 提醒只建立在 `v0.3` 已稳定 resident runtime 之上，不与常驻 MVP 同时推进。

**Test scenarios:**
- Happy path: 新的 `read=false` 信号到来时，能触发 popup 和系统通知。
- Edge case: 同一条未读重复轮询返回时，不重复制造新的提醒。
- Error path: 已读回写失败时，UI 恢复未读态并给出失败反馈。

**Verification:**
- 新信号从发现到处理的链路可演示、可恢复、不会重复刷屏。

- [ ] **Unit 5: 实现高级桌面行为状态机**

**Goal:** 让 widget 进入设计稿定义的桌面组件层级，而不只是一个固定常驻窄窗。

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

- **State truth:** 轮询结果、health、selectedGroupId、unread diff、popup queue、widget state 必须来自统一宿主状态，而不是窗口各自缓存。
- **Desktop differentiation:** `widget + tray` 是 `v0.3` 的核心，不应被提醒和高级行为提前吞没节奏。
- **Failure visibility:** 鉴权失败、限流、服务错误不仅要在主控台可见，也要在 widget/tray 体现。
- **Resident simplicity:** `v0.3` 常驻面只服务“当前组选中视图”，不扩张成第二套配置入口。
- **Platform risk:** click-through、dock、auto-hide 都有明显平台差异，必须在 `platform/*` 抽象层收口。
- **Carry cost control:** 当前阶段优先做一个 widget、一个 tray、一个 resident loop，避免过早引入多窗口矩阵。

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| 把 `v0.3` 做成“多窗口技术展示”而不是 daily-driver 产品切片 | 明确 `v0.3` 以 resident use value 为第一验收标准 |
| 在 resident MVP 中提前塞入 popup / notification，导致范围失控 | 坚持提醒闭环后置到 `v0.4` |
| 为了减少主控台依赖，过早把 group 切换塞进 tray/widget | 明确 `v0.3` 只消费 `selectedGroupId`，不新增切换入口 |
| 新增窗口后出现多处状态真相 | 继续以宿主共享状态为唯一数据来源 |
| 高级窗口行为过早进入主链路，导致桌面系统问题拖慢节奏 | `auto-hide/click-through` 独立为 `v0.5` |

## Documentation / Operational Notes

- 进入 `v0.3` 前，应准备一份 resident loop 验收清单，至少覆盖：
  - 主窗隐藏后 widget/tray 是否继续工作
  - tray 恢复主窗、暂停/恢复轮询是否稳定
  - `auth error / backoff / stale` 是否在 resident surface 上清晰可见
- 进入 `v0.4` 前，应准备 unread diff 与已读回写一致性检查清单，避免提醒链路一上来就重复刷屏。
- 若 `v0.3` 实现中发现 tray/widget 强烈需要 group switching，应该先回写路线图再扩 scope，而不是在代码里临时偷加。

## Sources & References

- Origin document: `prd.md`
- Supplemental requirements: `start.md`
- Related completed plans:
  - `docs/plans/2026-04-10-002-feat-watch-tower-v0-1-foundation-plan.md`
  - `docs/plans/2026-04-11-003-feat-watch-tower-v0-2-main-dashboard-plan.md`
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
