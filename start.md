## 服务器有以下接口:
使用 `x-api-key` 的方式鉴权, 放在请求头中:

```http
x-api-key: <your-open-signal-api-key>
```

缺少或错误的 `x-api-key` 时, 接口会返回 `401`.

1. `POST /api/open/watch-list/symbol-signals`

#### 1.1 请求体示例

```json
{
  "symbols": "BTCUSDT,ETHUSDT",
  "periods": "15m,1h",
  "signalTypes": "divMacd,vegas",
  "page": 1,
  "pageSize": 20
}
```

说明:
- `symbols`: 必填, 多个品种用逗号分隔
- `periods`: 可选, 多个周期用逗号分隔; 不传则查询所有周期
- `signalTypes`: 可选, 多个信号类型用逗号分隔; 不传则查询所有类型
- `page`: 可选, 默认 `1`
- `pageSize`: 可选, 默认 `20`, 最大 `100`

#### 1.2 响应示例

```json
{
  "total": 4,
  "page": 1,
  "pageSize": 20,
  "data": [
    {
      "symbol": "BTCUSDT",
      "period": "15m",
      "signals": {
        "divMacd": {
          "sd": 1,
          "t": 1704067200000,
          "read": false
        },
        "vegas": {
          "sd": -1,
          "t": 1704067100000,
          "read": true
        }
      },
      "t": 1704067200000
    }
  ]
}
```

字段说明:
- `sd`: side, `1=看多`, `-1=看空`
- `t`: 信号时间戳(毫秒)
- `read`: 是否已读, `false` 表示未读
- `signals`: 一个对象, key 为信号类型, 如 `divMacd`、`vegas`、`vegasT`、`atrIndex`、`tdMd`
- 注意: 信号只是一个逻辑上的 true/false, 没有数值

2. `POST /api/open/watch-list/symbol-alert/read-status`

#### 2.1 请求体示例

```json
{
  "symbol": "BTCUSDT",
  "period": "15m",
  "signalType": "divMacd",
  "read": true
}
```

说明:
- 用 `symbol + period + signalType` 定位一条信号
- `read=true` 表示标记为已读
- `read=false` 表示恢复为未读

#### 2.2 响应示例

```json
true
```

说明:
- 更新成功返回 `true`
- 目标记录或信号不存在时返回 `false`

3. `POST /api/open/watch-list/symbol-alert/delete-signal`

#### 3.1 请求体示例

```json
{
  "symbol": "BTCUSDT",
  "period": "15m",
  "signalType": "divMacd"
}
```

#### 3.2 响应示例

```json
true
```

说明:
- 删除成功返回 `true`
- 目标记录或信号不存在时返回 `false`

## 软件需求
 ### 1. 桌面端
 用户故事: 1. 可以在桌面端置顶显示 '10D', 'W','4D', '3D','2D', 'D','720', '480','360','240', '180', '120', '90', '60', '45', '30', '20', '15', '10', '8', '5', '4', '3', '2', '1',这些级别的某个具体(用户配置, 如vegas)信号;
 2. 用户可以隐藏到托盘, 或者悬浮隐藏在旁边位置
 3. 当检测到有新的 `read=false` 信号, 自动展开, UI上提醒用户, 还可以发送到系统通知(都可设置), 用户可以标记信号为已读
 4. 界面上的显示可以预定几个模板用来如何图形展示信号, 比如每个信号级别显示在最近60根k线的位置(就是60个竖条, 对应最近60根k线时间(可以根据时间级别算出来), 将信号触发的时间对应的竖条标记为一个醒目的背景色)
 5. 查询接口方案使用轮询, 比如1分钟一次, 可以设置
 ### 2. 移动端(后期再开发)
