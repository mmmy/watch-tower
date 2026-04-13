---
title: Watch Tower 桌面端迭代路线图
type: feat
status: active
date: 2026-04-13
revision: v0.6
origin: prd.md
---

# Watch Tower 桌面端迭代路线图（v0.6）

## Overview

本版路线图基于最新仓库现实重新校准后续节奏：

- `v0.1` 基座已完成。
- `v0.2` 主控台补强已完成。
- `v0.3` resident daily-driver MVP 已完成。
- `v0.4` minimal alert closure 已完成。
- `v0.5` desktop behavior upgrade 已完成。

因此当前最高杠杆问题，已经不再是：

- “产品能否主动提醒用户”
- “widget 能否成为真正低打扰、可唤醒的桌面组件”

而是：

**在提醒闭环和桌面行为都已成立后，下一步应如何把 Watch Tower 从个人日常工具推进到可对外试用的桌面产品。**

本次修订的结论是：

`已完成 v0.1/v0.2/v0.3/v0.4/v0.5 -> 多窗口编排与试用发布`

也就是说，下一阶段优先做的不是继续打磨单窗口 hover 细节，也不是把 tray/widget 扩成第二套主控台，而是用一个克制版本把以下能力补齐：

- 多 symbol 告警时的 popup 编排与去冲突
- 主控台中的未读恢复/回看入口
- 最小可用的打包、README 与试用 checklist
- 让外部试用用户能安装、理解并稳定体验当前桌面链路

## Revision Notes

相对 `v0.5` 版路线图，本次修订的关键变化如下：

- 将 `v0.4` 从“下一步提醒闭环”更新为**已完成阶段**，因为仓库与实施计划已证明 unread diff、单 popup、系统通知与已读回写链路已经落地。
- 将 `v0.5` 从“下一步桌面行为升级”更新为**已完成阶段**，因为仓库与实施计划已证明 `auto-hide / wake zone / hover / passive click-through fallback` 等桌面行为主链路已经落地。
- 将后续主轴从“提醒闭环优先”与“桌面行为优先”调整为**多窗口编排与试用发布优先**。
- 明确 `v0.6` 的目标不是继续重做提醒或 resident runtime，而是：
  - 多 symbol popup 复用与队列
  - 主控台未读恢复/回看入口
  - 可选市场总览 Grid
  - 安装包、README、默认权限与试用文档
- 继续把 tray/widget 内 group switching 留在后续版本之外，避免 resident 入口膨胀成第二套迷你控制台。

## Current State

当前仓库已经验证存在以下基础能力：

- 工程与宿主基础：
  - `src-tauri/` 已建立 Tauri v2 宿主结构
  - `src/` 已建立 React + Vite 前端入口
  - `src-tauri/src/config/*` 已有配置仓储
  - `src-tauri/src/polling/*` 已有信号请求、调度、backoff 与 unread diff 能力
  - `src-tauri/src/commands/mod.rs` 已提供前端与宿主命令边界
- 共享模型：
  - `src/shared/config-model.ts`
  - `src/shared/alert-model.ts`
  - `src/shared/period-utils.ts`
  - `src/shared/view-models.ts`
  - `src/shared/events.ts`
  - `src/shared/window-state.ts`
- 主控台能力：
  - `src/windows/main-dashboard/*` 已具备多 group 管理、当前组选中、布局预设、health 与 diagnostics
- resident runtime 能力：
  - `src-tauri/src/lib.rs` 已在启动期间初始化 resident surfaces
  - `src-tauri/src/windows/mod.rs` 已拦截主窗关闭并转为隐藏
  - `src-tauri/src/tray/mod.rs` 已支持 tray 状态同步、恢复主窗、暂停/恢复轮询与退出
  - `src-tauri/src/windows/edge_widget.rs` 已支持 widget 同步、隐藏与唤醒
- 提醒闭环能力：
  - `src-tauri/src/polling/unread_diff.rs` 已支持 unread diff
  - `src-tauri/src/windows/alert_popup.rs` 与 `src/windows/alert-popup/*` 已支持单活动 popup
  - `src-tauri/src/commands/mod.rs` 已支持已读回写、失败回滚与 dashboard focus intent
  - `docs/plans/2026-04-12-005-feat-watch-tower-v0-4-minimal-alert-closure-plan.md` 已标记为 `completed`
- 桌面行为能力：
  - `src-tauri/src/windows/hover_state.rs` 已支持 `passive / hover / interactive` 三态状态机
  - `src-tauri/src/platform/*` 已收口 click-through capability 与 fallback
  - `src/windows/edge-widget/hooks/use-edge-widget-events.ts` 已接入 pointer enter/leave 与 interaction intent
  - `src/windows/main-dashboard/components/diagnostics-panel.tsx` 已暴露 widget mode 与 capability
  - `docs/plans/2026-04-13-006-feat-watch-tower-v0-5-desktop-behavior-plan.md` 已标记为 `completed`

因此当前最重要的问题已经从：

- “当新的 `read=false` 信号出现时，产品能否可靠地主动提醒用户”
- “widget 是否具备足够稳定的低打扰桌面行为”

转向：

- “当多个 symbol 或多轮新信号同时出现时，产品能否有序编排提醒，而不是互相遮挡或反复打扰”
- “用户错过 popup 后，主控台里是否存在稳定的恢复入口”
- “当前产品是否已经具备外部试用所需的安装、说明与验收资料”

## Problem Frame

Watch Tower 的桌面差异化，第一阶段来自“常驻可扫一眼”；第二阶段来自“新信号会主动提醒你”；第三阶段来自“widget 已成为真正低打扰、可唤醒的桌面组件”。

但如果它现在停留在单 alert、单 popup、单人本地使用层，仍然会面临三个现实缺口：

- 多 symbol 告警并发时，现有提醒模型还缺少编排与恢复入口。
- 用户错过提醒后，主控台里还没有一个足够清晰的未读队列/恢复面板。
- 即使本地功能已成立，外部试用用户仍缺少安装包、README、默认权限说明和试用 checklist。

这意味着后续路线图要避免三个偏差：

- 把下一阶段做成“继续打磨 hover/click-through 细节”，却绕开多告警编排与试用发布这两个真正阻塞外部验证的问题。
- 为了追求一次性完整，把 `v0.6` 扩成完整通知中心、复杂静音策略、全平台一致分发体系，导致试用版本迟迟不能交付。
- 为了做多窗口编排，反而重建一套新的提醒状态层，破坏 `v0.4` 与 `v0.5` 已经建立的共享 snapshot 与 resident runtime。

所以这份路线图采用以下判断：

- **先把 `v0.6` 的多窗口编排与试用发布做成立**，验证 Watch Tower 已经不仅适合个人 daily-driver，也适合拿给外部试用用户。
- **市场总览 Grid 继续视作增强项**，只在不阻塞试用发布时进入 `v0.6`。
- **group switching 继续留在主控台**，避免 tray/widget 继续膨胀。

## Requirements Trace

- R1. 支持用户配置 `API Key`、监控分组、周期范围、信号类型、轮询频率和基础窗口策略。
- R2. 支持桌面端常驻，至少包含主控台、edge widget、tray controller 三类入口。
- R3. 支持按组展示 25 个周期的信号总览，并可查看单级别最近 60 根 K 线映射。
- R4. 当发现 `read=false` 的新信号时，支持提醒闭环：检测、通知、跳转、标记已读、失败回滚。
- R5. 轮询机制具备最小间隔保护、`401`/`429`/`5xx` 显式状态、退避和 stale 反馈。
- R6. 一个 group 内只承载一个 `symbol`，多组由主控台进行管理，resident surface 只展示当前 selected group。
- R7. 支持桌面高级行为：贴边、自动隐藏、观察态点击穿透或 fallback、hover 唤醒与状态机收敛。
- R8. 已完成的 `v0.1`、`v0.2`、`v0.3`、`v0.4`、`v0.5` 产物应被直接复用和延展，而不是在 `v0.6` 中重建另一套配置、轮询、resident、alert 或 widget behavior 状态层。
- R9. `v0.6` 需要支持多 symbol popup 编排、未读恢复入口和最小试用发布收口。
- R10. `v0.6` 不承担复杂静音策略、tray/widget group switching 或完整通知中心。

## Scope Boundaries

- 本路线图只覆盖桌面端，不包含移动端开发。
- 不引入历史信号回溯；服务端当前仍只返回“最新警报”。
- 不引入 WebSocket；数据同步仍以轮询为主。
- 不以“首版就做到跨平台完全一致”为目标；Windows 优先，macOS 在抽象层预留兼容位。
- `03 Optional Market Overview Grid` 继续作为增强视图，仅在不阻塞试用发布时进入 `v0.6`。
- `v0.6` 不重做 `v0.4` 的提醒闭环，也不重做 `v0.5` 的 widget behavior runtime。
- `v0.6` 不实现完整通知中心、复杂静音规则或历史提醒归档系统。
- `v0.6` 不在 tray/widget 中提供 group 切换；切换继续留在主控台。
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
  - tray/widget 仍可继续把 group orchestration 留在主控台，而不必发明常驻专用配置流
- `v0.3` 已完成的 resident 能力意味着：
  - 主窗隐藏后应用仍可继续常驻
  - widget 与 tray 已能消费共享运行态
- `v0.4` 已完成的提醒闭环意味着：
  - 新信号到来后，产品已经能进行 unread diff、提醒、跳转与已读回写
  - 当前 alert runtime 已经建立最小可靠闭环，不应在 `v0.6` 被推翻
- `v0.5` 已完成的桌面行为意味着：
  - widget 已具备 `passive / hover / interactive` 三态
  - 平台 click-through/fallback 与 diagnostics 已有明确语义
  - `v0.6` 不需要再把桌面行为本身当成主问题
- 当前仍未完成的关键差距是：
  - 多 symbol 告警时，popup 编排仍不够完整
  - 用户缺少主控台内的恢复/回看入口
  - 产品尚未具备最小试用发布所需的安装包、README 与试用说明

## Product Pressure Test

围绕“`v0.5` 已成立后下一步到底先做什么”，当前存在四条可选路径：

- **多窗口编排与试用发布优先**
  - 优点：
    - 直接补齐从“个人可用”到“可对外试用”的最后一道产品门槛
    - 能解决多 symbol 并发告警的真实使用问题
    - 能更快拿到外部用户对完整桌面链路的反馈
  - 风险：
    - 容易把 `v0.6` 扩成完整提醒中心或完整分发体系

- **继续桌面行为打磨优先**
  - 优点：widget 体验会更精致。
  - 问题：收益开始递减，而且无法补齐试用发布与多告警管理的缺口。

- **市场总览视图优先**
  - 优点：产品展示力会更强。
  - 问题：它更像增强视图，而不是当前最阻塞试用发布的关键路径。

- **控制面扩张优先**
  - 优点：tray/widget 会更像完整桌面控制器。
  - 问题：很容易让 resident 入口膨胀成第二套迷你主控台，反而增加维护成本。

结论：下一阶段应该明确押注 **多窗口编排与试用发布优先**，但采用**最小可用版本**推进，而不是一次性做完整通知中心或完整发布体系。

## Key Decisions

- 决策 1：将 `v0.1`、`v0.2`、`v0.3`、`v0.4`、`v0.5` 视为已完成阶段，后续路线图从已有主控台、resident runtime、alert runtime 和 widget behavior runtime 继续推进。
  - 理由：仓库与独立实施计划都已证明这五步不是未来工作。

- 决策 2：`v0.6` 以 **Orchestration & Trial Release** 为目标，而不是继续深化单 alert 或单 widget 行为。
  - 理由：当前最值得验证的是 Watch Tower 能否稳定处理多告警并被外部试用用户顺利安装体验。

- 决策 3：`v0.6` 只交付多 symbol popup 复用与队列、主控台恢复/回看入口、README/打包/试用 checklist，不带完整通知中心。
  - 理由：先把可交付试用版做成立，再进入更复杂的长期通知系统问题。

- 决策 4：`03 Optional Market Overview Grid` 继续作为 challenger scope，只在不阻塞主链路时进入 `v0.6`。
  - 理由：它有展示价值，但不应阻塞外部试用版本成形。

- 决策 5：`v0.6` 继续复用当前 `selectedGroupId`、`alert_runtime` 与 `widget_runtime`，不在 tray/widget 中承担 group switching。
  - 理由：减少 `v0.6` scope，避免 resident surface 演变成第二套控制面板。

- 决策 6：多窗口编排要建立在现有去重与已读回写语义之上，而不是平行造一套新的提醒状态机。
  - 理由：主控台、widget、tray、popup 的一致性不能靠各自推断。

- 决策 7：`v0.6` 的发布目标是“首个可对外试用版本”，而不是完整 GA 发布。
  - 理由：当前最需要的是更真实的外部反馈，而不是把分发体系一次性做满。

## High-Level Strategy

版本切分遵循四条原则：

- 每个版本都要形成一个可感知的用户价值闭环。
- `v0.6` 先验证“多告警可控 + 产品可试用”，再考虑更完整的通知中心或更大范围扩展。
- resident surface 继续优先服务“当前组选中视图”，不把 group orchestration 提前摊进 tray/widget。
- 只在现有基座、主控台、resident runtime、alert runtime 和 widget behavior runtime 上增量推进，不为了“更干净的未来架构”牺牲当前产品节奏。

## Version Roadmap

| Version | Theme | Scope | Release bar |
|------|------|------|------|
| v0.1 | 基座与验证壳 | Tauri 工程、配置持久化、轮询、共享模型、25 周期与 60-bar 验证壳 | 已完成，作为后续所有版本的共享基础 |
| v0.2 | 主控台补强版 | 多组配置、当前组选中、窗口策略、布局预设、主控台产品化 | 已完成，作为 resident runtime 的配置与状态来源 |
| v0.3 | Resident Daily-Driver MVP | 单 edge widget、tray controller、主窗隐藏后继续常驻、health/backoff/auth 状态外显 | 已完成，不开主控台也能通过 widget + tray 持续监控当前 group |
| v0.4 | Minimal Alert Closure MVP | unread diff、单 symbol popup、系统通知、单一通知开关、乐观已读回写、失败回滚、去重 | 已完成，产品已具备最小可靠提醒闭环 |
| v0.5 | Desktop Behavior Upgrade | auto-hide、wake zone、hover 唤醒、观察态 click-through 或 fallback、widget 状态机 | 已完成，widget 已具备真正桌面组件的交互层级 |
| v0.6 | Orchestration & Trial Release | 多 symbol popup 编排、可选总览 Grid、恢复面板、打包发布与试用文档 | 首个可对外试用版本可被安装、理解并稳定体验 |

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

### v0.3 Resident Daily-Driver MVP（已完成）

**目标:** 让 Watch Tower 首次具备“主窗关闭后仍值得常驻桌面”的日常使用价值。

**当前结果:**
- 已具备单个 edge widget 窗口
- 已具备 tray controller
- 已打通主窗隐藏/恢复与 resident runtime 生命周期
- 已实现轮询状态、backoff、鉴权失败、stale 状态在 widget/tray 中的基础外显
- 已实现主控台当前 selected group 与 widget 同步

### v0.4 Minimal Alert Closure MVP（已完成）

**目标:** 在 resident loop 的基础上，把“发现新信号并处理”的最小闭环真正跑通。

**当前结果:**
- 已具备 unread diff 检测
- 已具备单活动 popup 与系统通知
- 已具备单一开关级别的通知控制
- 已具备标记已读、乐观更新与失败回滚
- 已具备从 popup 跳转到主控台对应详情
- 已具备同一条未读去重，避免重复提醒

### v0.5 Desktop Behavior Upgrade（已完成）

**目标:** 把 widget 从“可用的常驻窄窗”升级为真正贴边、可隐藏、可唤醒的桌面组件。

**当前结果:**
- 已具备 `passive / hover / interactive` 三态状态机
- 已具备 auto-hide、wake zone 与 hover reveal 主链路
- 已具备观察态 click-through capability 与显式 fallback
- 已具备 alert 拉起与 idle 回落协同
- 已具备 diagnostics 中的 widget mode / capability 可见性

### v0.6 Orchestration & Trial Release

**目标:** 在已有常驻、提醒和桌面行为能力上，解决多 symbol 并发提醒与外部试用交付问题。

**包含范围:**
- 多 symbol popup 复用与队列
- popup 可见上限与优先级管理
- 主控台未读队列 / 恢复面板
- 可选市场总览 Grid
- 安装包、README、默认权限与试用 checklist

**不包含:**
- 完整通知中心
- 复杂静音策略
- tray/widget 内 group switching
- 重做 `v0.4` alert runtime
- 重做 `v0.5` widget behavior runtime

**版本验收:**
- 多 symbol 同时告警不会互相遮挡或无限刷屏
- 用户错过 popup 后，仍能在主控台稳定回看和处理未读
- 应用可以交付给外部试用用户安装和体验
- README、默认权限与试用 checklist 足以支持一次真实试用

## Implementation Units

- [x] **Unit 1: 完成基座与验证壳**

**Goal:** 建立桌面宿主、共享模型、配置仓储、轮询/退避与主窗口验证壳。

**Status:** 已完成，对应 `docs/plans/2026-04-10-002-feat-watch-tower-v0-1-foundation-plan.md`

- [x] **Unit 2: 将验证壳提升为可运营主控台**

**Goal:** 让主窗口从“单组验证工具”升级为 resident runtime 可依赖的产品主控台。

**Status:** 已完成，对应 `docs/plans/2026-04-11-003-feat-watch-tower-v0-2-main-dashboard-plan.md`

- [x] **Unit 3: 交付 resident daily-driver MVP**

**Goal:** 让应用具备主控台之外的长期常驻入口，并验证“当前组选中 + widget + tray”已足够支撑日常桌面使用。

**Status:** 已完成，对应 `docs/plans/2026-04-11-004-feat-watch-tower-v0-3-resident-mvp-plan.md`

- [x] **Unit 4: 打通最小提醒闭环**

**Goal:** 把新信号从“被轮询发现”推进到“被用户看到、处理并回写”的最小可靠流程。

**Status:** 已完成，对应 `docs/plans/2026-04-12-005-feat-watch-tower-v0-4-minimal-alert-closure-plan.md`

- [x] **Unit 5: 实现高级桌面行为状态机**

**Goal:** 让 widget 进入设计稿定义的桌面组件层级，而不只是一个固定常驻窄窗。

**Status:** 已完成，对应 `docs/plans/2026-04-13-006-feat-watch-tower-v0-5-desktop-behavior-plan.md`

- [ ] **Unit 6: 完成多窗口编排与试用发布收口**

**Goal:** 在已有常驻、提醒和桌面行为能力上，解决多 symbol 并发提醒与试用交付问题。

**Requirements:** R2, R4, R5, R7, R8, R9, R10

**Dependencies:** Unit 4, Unit 5

**Likely files:**
- Modify: `src-tauri/src/windows/alert_popup.rs`
- Create: `src-tauri/src/windows/queue.rs`
- Create: `src/windows/main-dashboard/components/unread-queue.tsx`
- Create: `src/windows/main-dashboard/components/recovery-panel.tsx`
- Create: `src/windows/market-overview/index.tsx`
- Modify: `src/windows/main-dashboard/index.tsx`
- Modify: `src/shared/view-models.ts`
- Modify: `src/shared/events.ts`
- Modify: `src-tauri/tauri.conf.json`
- Create: `README.md`
- Create: `docs/checklists/v0-6-trial-release-acceptance.md`
- Test: `src/windows/main-dashboard/components/unread-queue.test.tsx`
- Test: `src/windows/market-overview/index.test.tsx`

**Approach:**
- 以 `symbol` 作为 popup 复用键；同 symbol 更新已有 popup，不重复开窗。
- visible popup 数量设上限，超出部分进入队列。
- 主控台提供最小未读恢复/回看入口，而不是把恢复逻辑散落在 popup 消失后的空白地带。
- 打包、README 与试用 checklist 以“让外部用户能安装并理解产品”为目标，而不是一次性完成所有发布治理。
- Grid 只在不阻塞试用发布的前提下补齐。

**Test scenarios:**
- Happy path: 不同 symbol 短时间连续告警时，popup 按优先级进入可见栈或等待队列。
- Happy path: 用户错过 popup 后，仍能在主控台恢复队列中进入对应详情并处理未读。
- Edge case: 屏幕高度不足时，低优先级 popup 进入队列而不是互相遮挡。
- Edge case: backoff 或 paused 状态期间不会无限制造普通级别 popup。
- Error path: 安装包与 README 缺少必要权限说明时，应在试用 checklist 中被明确拦截。

**Verification:**
- 多 symbol 告警管理可控、不重叠、不刷屏。
- 错过提醒后仍存在稳定恢复路径。
- 应用具备外部试用所需的打包与文档基础。

## System-Wide Impact

- **State truth:** 轮询结果、health、selectedGroupId、alert runtime、widget runtime、popup queue 与恢复入口必须来自统一宿主状态，而不是窗口各自缓存。
- **Alert concurrency:** `v0.6` 的最大新增复杂度来自多 symbol 并发提醒；任何编排都必须建立在现有去重与已读回写语义之上。
- **Desktop continuity:** `widget + tray + popup + dashboard` 已构成完整桌面链路，`v0.6` 的任务是让这条链路在多告警情况下仍然可解释、可恢复、可试用。
- **Failure visibility:** 鉴权失败、限流、服务错误、fallback 能力与队列堆积不仅要在主控台可见，也要在 popup/widget/tray 里保持一致心智。
- **Resident simplicity:** tray/widget 继续只服务当前组选中与最小会话控制，不扩张成第二套配置入口。
- **Carry cost control:** 当前阶段先做一个编排模型、一条恢复入口、一个试用发布闭环，再考虑更大的通知系统与发布体系。

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| 在 `v0.6` 中提前做完整通知中心，导致首个试用版本迟迟不能交付 | 坚持先做 popup 复用、队列、恢复入口与试用文档 |
| 为了支持多窗口编排，平行造第二套 alert runtime | 继续以共享 snapshot 与现有 alert runtime 为唯一提醒真相来源 |
| 为了减少主控台依赖，过早把 group 切换塞进 tray/widget | 明确 `v0.6` 继续只消费 `selectedGroupId`，不新增切换入口 |
| 市场总览 Grid 抢占主链路，拖慢试用发布 | 将 Grid 视作 challenger scope，不阻塞安装包、README 与恢复入口 |
| 发布收口只做打包不做说明，导致外部试用反馈失真 | README、默认权限说明与试用 checklist 必须作为 `v0.6` 一部分交付 |

## Documentation / Operational Notes

- 进入 `v0.6` 执行前，应准备一份编排与试用发布验收清单，至少覆盖：
  - 多 symbol popup 是否复用与排队正确
  - 主控台未读恢复/回看入口是否直接可用
  - popup 去重与已读回写语义是否保持与 `v0.4` 一致
  - 安装包、README 与默认权限说明是否足以支持外部试用
- 若 `v0.6` 实现中发现 popup 编排必须重写 alert runtime，应该先回写路线图再扩 scope，而不是在代码里临时偷换状态来源。
- 若 Grid 无法在不影响试用版本节奏的前提下落地，应允许把它保留为 `v0.6` 的可选增强项，而不是阻塞试用发布。

## Sources & References

- Origin document: `prd.md`
- Supplemental requirements: `start.md`
- Related completed plans:
  - `docs/plans/2026-04-10-002-feat-watch-tower-v0-1-foundation-plan.md`
  - `docs/plans/2026-04-11-003-feat-watch-tower-v0-2-main-dashboard-plan.md`
  - `docs/plans/2026-04-11-004-feat-watch-tower-v0-3-resident-mvp-plan.md`
  - `docs/plans/2026-04-12-005-feat-watch-tower-v0-4-minimal-alert-closure-plan.md`
  - `docs/plans/2026-04-13-006-feat-watch-tower-v0-5-desktop-behavior-plan.md`
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
