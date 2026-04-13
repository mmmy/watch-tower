# Watch Tower v0.5 桌面行为手测清单

## 测试准备

- [ ] 使用真实桌面运行环境测试，而不是浏览器预览。
- [ ] 准备至少一个能正常轮询的有效监控组。
- [ ] 确保同一会话中主控台、edge widget 和 alert popup 链路都可用。
- [ ] 先记录当前测试环境：
  - 操作系统：
  - 运行模式：dev / packaged
  - 停靠方向：left / right

## 基础冒烟检查

1. 使用已保存的有效配置启动应用。
2. 确认主控台能正常打开。
3. 确认 edge widget 已创建，并出现在当前配置的停靠边。
4. 确认轮询处于运行状态，widget 仍显示当前 selected group 的快照。

预期结果：
- 主控台可正常使用。
- widget 可见且位置正确。
- 不会一启动就出现 widget behavior runtime 相关诊断错误。

## Passive 到 Hover 到 Interactive

1. 将鼠标移开，让 widget 有机会回到 `passive`。
2. 等待足够时间，让 widget 退回低打扰状态。
3. 将鼠标移入 widget 边缘 / 唤醒区域。
4. 观察 widget 是否被唤醒并显露。
5. 在 widget 内部点击一次。

预期结果：
- widget 能回到 `passive`，不会永久消失。
- 鼠标进入唤醒区域时，widget 会显露出来。
- 在 widget 内点击后，会进入明确的 `interactive` 状态。
- `interactive` 状态下 widget 保持可点击。

## Interactive 回到 Passive

1. 让 widget 进入 `interactive`。
2. 停止移动鼠标，并把鼠标移出 widget。
3. 等待 idle timeout 到期。

预期结果：
- widget 不会因为轻微鼠标抖动立刻闪退。
- idle timeout 后会回到 `passive`。
- 如果当前 placement 支持 auto-hide，会干净地退回隐藏位置。
- 状态切换后 resident snapshot 仍然保留。

## 停靠边几何验证

### 右侧停靠

1. 将 dock side 设置为 `right`。
2. 保存配置并等待 widget 同步。
3. 重复一次 `passive -> hover -> interactive -> passive`。

预期结果：
- visible placement 贴住右边缘。
- hidden placement 只留下可唤醒的窄边。
- 显露和隐藏过程中不会发生垂直跳动。

### 左侧停靠

1. 将 dock side 设置为 `left`。
2. 保存配置并等待 widget 同步。
3. 重复一次 `passive -> hover -> interactive -> passive`。

预期结果：
- visible placement 贴住左边缘。
- hidden placement 能正确镜像。
- top offset 在显露 / 隐藏过程中保持稳定。

## Alert 拉起行为

1. 先让 widget 回到 `passive`。
2. 触发或等待一个新的 alert 到来。
3. 观察 widget 和 popup 的行为。
4. 如果可能，让同一条 alert 在后续轮询周期继续存在。

预期结果：
- 新 alert 可以把 `passive` widget 拉起到一次可见的 `interactive`。
- 现有 popup 行为仍然正常。
- 同一条 alert 不会在 widget 已经 interactive 时持续重复拉起。
- alert 拉起不会破坏 `v0.4` 的 dedupe 行为。

## 运行时健康状态

### Paused

1. 从 tray 或主控台暂停轮询。
2. 再次尝试唤醒 widget。

预期结果：
- widget 仍然可以被唤醒并交互。
- diagnostics 会显示 paused，而不会把 widget behavior state 搞乱。

### Backoff / Request Error

1. 模拟或制造 backoff / request error。
2. 重复唤醒流程。

预期结果：
- widget 仍然可以恢复。
- diagnostics 能解释当前 degraded runtime。
- widget 不会卡在“看不见”或“不可交互”状态。

### Stale

1. 让 snapshot 进入 stale，或模拟 stale 状态。
2. 重复唤醒流程。

预期结果：
- widget 仍然可以被唤醒。
- footer / diagnostics 能正确解释 stale。

## Click-Through 与 Fallback

1. 让 widget 回到 `passive`。
2. 尝试与 widget 下方或周围的桌面区域交互。
3. 然后再唤醒 widget，并直接与它交互。
4. 查看 dashboard diagnostics 中的 capability / fallback 文案。

预期结果：
- 在支持 click-through 的平台上，`passive` 应该表现为更低打扰。
- 在不支持的平台上，widget 仍然应该可恢复、可使用。
- diagnostics 会明确说明 fallback，而不是静默失败。
- 不能出现“widget 已隐藏且无法再唤醒”的死态。

## 回归保护

1. 从 popup 打开 alert 对应的 dashboard。
2. 标记一条 alert 为已读。
3. 在主控台切换 selected group。
4. 从 tray 恢复主控台。

预期结果：
- `v0.4` 的 popup 和已读闭环仍然正常。
- `selectedGroupId` 语义没有变化。
- tray restore 仍然可用。
- widget 行为升级不会把 widget 或 tray 变成第二套控制台。

## 验收结论

- [ ] `passive -> hover -> interactive -> passive` 在当前机器上成立。
- [ ] 左右停靠几何表现符合预期。
- [ ] alert 拉起 widget 一次且不破坏 popup 行为。
- [ ] paused / backoff / stale 状态下仍可恢复。
- [ ] 当前平台上的 click-through 或 fallback 可理解。
- [ ] 没有观察到死态或无法唤醒的 widget。

## 备注

- 发现的问题：
- 需要继续调参的地方：
- 平台相关注意事项：
