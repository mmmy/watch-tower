---
title: Watch Tower 桌面端迭代路线图
type: feat
status: active
date: 2026-04-12
revision: v0.5
origin: prd.md
---

# Watch Tower 桌面端迭代路线图（v0.5）

## Overview

本版路线图基于最新仓库现实重新校准后续节奏：

- `v0.1` 基座已完成。
- `v0.2` 主控台补强已完成。
- `v0.3` resident daily-driver MVP 也已完成，仓库已具备 `widget + tray + close-to-hide + pause/resume + resident health` 的常驻主链路。

因此当前最高杠杆问题，已经不再是“如何让桌面端常驻成立”，而是：

**在 resident 已成立后，下一步应该优先补上什么，才能让 Watch Tower 从被动扫一眼工具，变成真正值得每天开着的桌面产品。**

本次修订的结论是：

`已完成 v0.1/v0.2/v0.3 -> 最小提醒闭环 -> 高级桌面行为 -> 多窗口编排与试用发布`

也就是说，下一阶段优先做的不是 `auto-hide / hover wake / click-through`，也不是把 tray/widget 扩成第二套小控制台，而是用一个克制版本把“发现新信号 -> 提醒 -> 处理 -> 回写”的主价值闭环跑通。

## Revision Notes

相对 `v0.4` 版路线图，本次修订的关键变化如下：

- 将 `v0.3` 从“下一步 resident MVP”更新为**已完成阶段**，因为仓库与独立实施计划已证明这一步已经落地。
- 将后续主轴从“继续深化 resident 本身”调整为**最小提醒闭环优先**。
- 明确 `v0.4` 的目标不是“完整多弹窗系统”，而是：
  - unread diff 检测
  - 单 symbol popup
  - 系统通知
  - 单一开关级别的通知控制
  - 已读回写与失败回滚
  - 去重，避免同一未读重复刷屏
- 继续把 `auto-hide / hover wake / click-through` 留在 `v0.5`，避免高级桌面行为抢占提醒闭环的主链路。
- 继续把 tray/widget 内 group switching 留在后续版本，避免 resident 入口膨胀成第二套迷你控制台。

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
  - `src/windows/main-dashboard/*` 已具备多 group 管理、当前组选中、布局预设、health 与 diagnostics
- resident runtime 能力：
  - `src-tauri/src/lib.rs` 已在启动期间初始化 resident surfaces
  - `src-tauri/src/windows/mod.rs` 已拦截主窗关闭并转为隐藏
  - `src-tauri/src/tray/mod.rs` 已支持 tray 状态同步、恢复主窗、暂停/恢复轮询与退出
  - `docs/plans/2026-04-11-004-feat-watch-tower-v0-3-resident-mvp-plan.md` 已标记为 `completed`

因此当前最重要的问题已经从：

- “主窗关闭后产品是否仍有独立价值”

转向：

- “当新的 `read=false` 信号出现时，产品能否可靠地主动提醒用户，并让用户完成处理闭环”

## Problem Frame

Watch Tower 的桌面差异化，第一阶段来自于“常驻可扫一眼”；但如果它始终停留在被动查看层，用户仍然需要主动盯住 widget 才能获得价值。

在 resident runtime 已成立后，当前最该验证的下一层产品价值是：

- 新信号出现时，用户是否能被可靠提醒，而不是只能自己碰巧扫到。
- 提醒是否足够克制，不会因为重复未读或临时失败而刷屏。
- 用户处理提醒后，已读状态是否能稳定回写，并在失败时保持一致性。
- 这一闭环是否可以建立在现有共享状态与 resident surfaces 之上，而不是再造一套提醒专用状态模型。

这意味着后续路线图要避免三个偏差：

- 把下一阶段做成“高级桌面行为展示”，却还没有补上主动提醒这个核心价值缺口。
- 为了追求一次性完整，过早引入多 popup 编排、复杂队列和大量动画，导致提醒闭环本身反而迟迟不能成立。
- 为了减少主控台依赖，提前把 group switching、静音策略和更多控制入口堆进 tray/widget，导致提醒 MVP scope 膨胀。

所以这份路线图采用以下判断：

- **先把最小提醒闭环做成立**，验证 Watch Tower 不只是能被动展示状态，也能主动传递新信号。
- **高级桌面行为后置到 `v0.5`**，建立在提醒价值成立之后。
- **group switching 继续留在主控台**，避免 tray/widget 提前演变成第二套控制面板。

## Requirements Trace

- R1. 支持用户配置 `API Key`、监控分组、周期范围、信号类型、轮询频率和基础窗口策略。
- R2. 支持桌面端常驻，至少包含主控台、edge widget、tray controller 三类入口。
- R3. 支持按组展示 25 个周期的信号总览，并可查看单级别最近 60 根 K 线映射。
- R4. 当发现 `read=false` 的新信号时，支持提醒闭环：检测、通知、跳转、标记已读、失败回滚。
- R5. 轮询机制具备最小间隔保护、`401`/`429`/`5xx` 显式状态、退避和 stale 反馈。
- R6. 一个 group 内只承载一个 `symbol`，多组由主控台进行管理，resident surface 只展示当前 selected group。
- R7. 后续支持桌面高级行为：贴边、自动隐藏、观察态点击穿透、hover 唤醒、popup 复用与队列。
- R8. 已完成的 `v0.1`、`v0.2`、`v0.3` 产物应被直接复用和延展，而不是在后续版本中重建另一套配置、轮询、resident 或提醒状态层。
- R9. `v0.4` 提醒 MVP 不承担高级桌面行为、group switching 或多 popup 编排；它只负责把当前 resident 产品推进到可靠的提醒闭环。

## Scope Boundaries

- 本路线图只覆盖桌面端，不包含移动端开发。
- 不引入历史信号回溯；服务端当前仍只返回“最新警报”。
- 不引入 WebSocket；数据同步仍以轮询为主。
- 不以“首版就做到跨平台完全一致”为目标；Windows 优先，macOS 在抽象层预留兼容位。
- `03 Optional Market Overview Grid` 继续作为增强视图，不进入 `v0.4` 主链路。
- `v0.4` 不实现 `auto-hide / hover wake / click-through`。
- `v0.4` 不实现多 popup 队列编排或复杂可见上限策略。
- `v0.4` 不实现复杂静音策略；只提供最小可用的通知开关。
- `v0.4` 不在 tray/widget 中提供 group 切换；切换继续留在主控台。
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
  - 下一阶段不需要再验证 resident loop 是否存在，而要验证提醒闭环是否成立
- 当前仍未完成的关键差距是：
  - 新信号到来后，缺少稳定的 unread diff 与提醒入口
  - 用户尚不能从提醒直接完成处理并回写已读状态
  - 产品仍偏向“被动查看”，尚未形成主动通知价值

## Product Pressure Test

围绕“resident 已成立后下一步到底先做什么”，当前存在四条可选路径：

- **提醒闭环优先**
  - 优点：
    - 直接补齐“主动提醒”这个最核心的价值缺口
    - 更快验证 Watch Tower 是否值得真的每天开着
    - 能直接建立从检测到处理的用户闭环
  - 风险：
    - 容易被多弹窗编排和通知 polish 吸走 scope

- **桌面行为优先**
  - 优点：resident 体验会更像真正桌面组件。
  - 问题：更偏体验增强，而不是补上主动提醒的主价值缺口。

- **运营控制优先**
  - 优点：tray/widget 会更像完整桌面控制器。
  - 问题：很容易让 resident 入口膨胀成第二套迷你主控台。

- **发布试用优先**
  - 优点：更快拿到外部反馈。
  - 问题：如果提醒闭环还没成立，外部用户最先感受到的仍会是核心能力缺口。

结论：下一阶段应该明确押注 **提醒闭环优先**，但采用**最小可用版本**推进，而不是一次性解决完整提醒系统。

## Key Decisions

- 决策 1：将 `v0.1`、`v0.2`、`v0.3` 视为已完成阶段，后续路线图从已有主控台、resident runtime 和共享状态继续推进。
  - 理由：仓库与完成计划都已证明这三步不是未来工作。

- 决策 2：`v0.4` 以 **最小提醒闭环 MVP** 为目标，而不是“多窗口提醒架构完整”或“高级桌面行为完整”。
  - 理由：当前最值得验证的是 Watch Tower 是否具备主动提醒的新价值层。

- 决策 3：`v0.4` 只交付 unread diff、单 symbol popup、系统通知、已读回写与失败回滚，不带复杂多 popup 编排。
  - 理由：先把提醒本身做可靠，再进入编排与扩展问题。

- 决策 4：`v0.4` 的系统通知只要求单一开关级别的控制，不在这一版引入更复杂的静音策略。
  - 理由：提醒 MVP 需要避免噪音，但不应因此提前膨胀出完整通知中心。

- 决策 5：`v0.4` 继续复用当前主控台 `selectedGroupId` 与共享状态，不在 tray/widget 中承担 group switching。
  - 理由：减少提醒 MVP scope，避免 tray/widget 演变成第二套迷你控制台。

- 决策 6：高级桌面行为继续拆到 `v0.5`。
  - 理由：`auto-hide`、`hover wake`、`click-through` 会显著增加平台行为复杂度，但并不直接补上提醒主价值。

- 决策 7：多 popup 复用、队列和发布收口继续拆到 `v0.6`。
  - 理由：这是提醒系统扩展与外部试用问题，不应阻塞最小闭环成立。

- 决策 8：共享 store、轮询状态、resident 视图与提醒状态继续维持单一真相来源。
  - 理由：主控台、widget、tray、popup 的一致性不能靠各自推断。

## High-Level Strategy

版本切分遵循四条原则：

- 每个版本都要形成一个可感知的用户价值闭环。
- `v0.4` 先验证“产品会主动叫你”，再考虑“桌面交互是否更丝滑”。
- resident surface 继续优先服务“当前组选中视图”，不把 group orchestration 提前摊进 tray/widget。
- 只在现有基座、主控台和 resident runtime 上增量推进，不为了“更干净的未来架构”牺牲当前产品节奏。

## Version Roadmap

| Version | Theme | Scope | Release bar |
|------|------|------|------|
| v0.1 | 基座与验证壳 | Tauri 工程、配置持久化、轮询、共享模型、25 周期与 60-bar 验证壳 | 已完成，作为后续所有版本的共享基础 |
| v0.2 | 主控台补强版 | 多组配置、当前组选中、窗口策略、布局预设、主控台产品化 | 已完成，作为 resident runtime 的配置与状态来源 |
| v0.3 | Resident Daily-Driver MVP | 单 edge widget、tray controller、主窗隐藏后继续常驻、health/backoff/auth 状态外显 | 已完成，不开主控台也能通过 widget + tray 持续监控当前 group |
| v0.4 | Minimal Alert Closure MVP | unread diff、单 symbol popup、系统通知、单一通知开关、乐观已读回写、失败回滚、去重 | 新信号能被发现、提醒、处理并同步，且不会重复刷屏 |
| v0.5 | Desktop Behavior Upgrade | auto-hide、wake zone、hover 唤醒、观察态 click-through、widget 状态机 | widget 具备真正桌面组件的交互层级 |
| v0.6 | Orchestration & Trial Release | 多 symbol popup 编排、可选总览 Grid、恢复面板、打包发布与试用文档 | 可作为首个对外试用版本分发 |

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

### v0.4 Minimal Alert Closure MVP

**目标:** 在已有 resident loop 的基础上，把“发现新信号并处理”的最小闭环真正跑通。

**包含范围:**
- unread diff 检测
- 单 symbol slide-out popup
- 系统通知
- 单一开关级别的通知控制
- 标记已读
- 乐观更新与失败回滚
- 从 popup 跳转到主控台对应详情
- 同一条未读去重，避免重复提醒

**不包含:**
- 多 popup 编排
- popup 可见上限策略
- 复杂静音策略
- auto-hide
- click-through
- hover wake
- tray/widget 内 group switching

**版本验收:**
- 新的 `read=false` 信号到来时，能可靠触发 popup 和系统通知
- 同一条未读在重复轮询返回时不会重复刷屏
- 用户处理已读后，状态能稳定回写；失败时会恢复并给出反馈
- 从提醒进入主控台详情的路径是直接且一致的

### v0.5 Desktop Behavior Upgrade

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

### v0.6 Orchestration & Trial Release

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

- [x] **Unit 3: 交付 resident daily-driver MVP**

**Goal:** 让应用具备主控台之外的长期常驻入口，并验证“当前组选中 + widget + tray”已足够支撑日常桌面使用。

**Status:** 已完成，对应 `docs/plans/2026-04-11-004-feat-watch-tower-v0-3-resident-mvp-plan.md`

- [ ] **Unit 4: 打通最小提醒闭环**

**Goal:** 把新信号从“被轮询发现”推进到“被用户看到、处理并回写”的最小可靠流程。

**Requirements:** R4, R5, R8, R9

**Dependencies:** Unit 1, Unit 2, Unit 3

**Likely files:**
- Create: `src/shared/unread-diff.ts`
- Create: `src-tauri/src/windows/alert_popup.rs`
- Create: `src/windows/alert-popup/index.tsx`
- Create: `src/windows/alert-popup/components/alert-card.tsx`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/polling/alerts_client.rs`
- Modify: `src-tauri/src/polling/scheduler.rs`
- Modify: `src/shared/events.ts`
- Modify: `src-tauri/src/app_state.rs`
- Test: `src/shared/unread-diff.test.ts`
- Test: `src/windows/alert-popup/components/alert-card.test.tsx`

**Approach:**
- unread diff 以“本轮快照 vs 上轮快照”为准，而不是基于 UI 自己缓存推断。
- 先做单 symbol popup，不提前解决多弹窗编排。
- 已读回写采用乐观更新 + 后台失败回滚。
- popup 与系统通知要共享同一份去重判断，避免同一条未读被重复打扰。
- 前端触发的已读写回继续走宿主命令边界，避免在 resident surfaces 中各自直连接口。

**Test scenarios:**
- Happy path: 新的 `read=false` 信号到来时，能触发 popup 和系统通知。
- Happy path: 用户从 popup 进入主控台详情并标记已读后，状态完成回写。
- Edge case: 同一条未读重复轮询返回时，不重复制造新的提醒。
- Error path: 已读回写失败时，UI 恢复未读态并给出失败反馈。

**Verification:**
- 新信号从发现到处理的链路可演示、可恢复、不会重复刷屏。
- Watch Tower 从“被动扫一眼”升级为“能主动提醒”的桌面产品。

- [ ] **Unit 5: 实现高级桌面行为状态机**

**Goal:** 让 widget 进入设计稿定义的桌面组件层级，而不只是一个固定常驻窄窗。

**Requirements:** R2, R4, R7, R8

**Dependencies:** Unit 4

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
- Modify: `src-tauri/src/windows/alert_popup.rs`
- Create: `src-tauri/src/windows/queue.rs`
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

- **State truth:** 轮询结果、health、selectedGroupId、unread diff、popup state、widget state 必须来自统一宿主状态，而不是窗口各自缓存。
- **Desktop differentiation:** `widget + tray` 已经建立 resident 基座，下一步要验证的是主动提醒价值，而不是只继续打磨存在感。
- **Failure visibility:** 鉴权失败、限流、服务错误不仅要在主控台可见，也要在 widget/tray/popup 体现。
- **Resident simplicity:** tray/widget 继续只服务当前组选中与最小会话控制，不扩张成第二套配置入口。
- **Platform risk:** click-through、dock、auto-hide 都有明显平台差异，必须在 `platform/*` 抽象层收口。
- **Carry cost control:** 当前阶段先做一个 popup 模型、一个去重闭环，再进入多窗口矩阵与试用收尾。

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| 在 `v0.4` 中提前塞入多 popup 编排，导致提醒闭环本身迟迟不能落地 | 坚持先做单 symbol popup + 去重 + 回写 |
| 为了减少主控台依赖，过早把 group 切换塞进 tray/widget | 明确 `v0.4` 继续只消费 `selectedGroupId`，不新增切换入口 |
| unread diff 与 UI 自己缓存各算一套，造成重复提醒或漏提醒 | 继续以宿主共享状态为唯一提醒判定来源 |
| 已读回写失败后 UI 与服务端状态分叉 | 使用乐观更新 + 失败回滚，并保留明确错误反馈 |
| 高级桌面行为过早进入主链路，导致平台行为问题拖慢提醒交付 | `auto-hide/click-through` 独立为 `v0.5` |

## Documentation / Operational Notes

- 进入 `v0.4` 前，应准备一份提醒闭环验收清单，至少覆盖：
  - unread diff 是否稳定识别新信号
  - popup / 系统通知是否去重
  - 已读回写成功与失败回滚是否一致
  - 从提醒跳转主控台详情是否直接可用
- 进入 `v0.5` 前，应准备平台行为风险清单，避免在提醒闭环未稳定前就引入 `auto-hide` 与 `click-through` 复杂度。
- 若 `v0.4` 实现中发现 tray/widget 强烈需要 group switching，应该先回写路线图再扩 scope，而不是在代码里临时偷加。

## Sources & References

- Origin document: `prd.md`
- Supplemental requirements: `start.md`
- Related completed plans:
  - `docs/plans/2026-04-10-002-feat-watch-tower-v0-1-foundation-plan.md`
  - `docs/plans/2026-04-11-003-feat-watch-tower-v0-2-main-dashboard-plan.md`
  - `docs/plans/2026-04-11-004-feat-watch-tower-v0-3-resident-mvp-plan.md`
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
