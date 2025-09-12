# Glass Dashboard+ · 接口与集成指南

本文描述首页各面板的数据来源、接口契约、示例，以及预留“日志面板”的设计方案。HTML 页面无提示文案，所有规范以本文为准。

## 目录

- [面板与接口一览](#面板与接口一览)
- [核心接口契约](#核心接口契约)
    - [/ （根路径）](#根路径)
    - [/healthy](#healthy)
    - [/status](#status)
    - [/actuator/info 或 /version](#actuatorinfo-或-version)
    - [依赖面板（可选）](#依赖面板可选)
    - [静态资源热身（可选）](#静态资源热身可选)
- [前端可配置项（Query）](#前端可配置项query)
- [持久化与本地存储](#持久化与本地存储)
- [一键压测说明](#一键压测说明)
- [预留：日志面板设计](#预留日志面板设计)
    - [API 设计](#api-设计)
    - [消息格式](#消息格式)
    - [后端实现建议（Kotlin + Spring Boot）](#后端实现建议kotlin--spring-boot)
    - [安全与配额](#安全与配额)

---

## 面板与接口一览

| 面板       | 方法  | 路径（默认）           | 说明/判定                                      |
|----------|-----|------------------|--------------------------------------------|
| 外网可达     | GET | `/`              | `2xx/3xx` 视为 OK，记录延迟                       |
| 后端就绪     | GET | `/healthy`       | `2xx` 视为 OK，记录延迟                           |
| Redis 连接 | GET | `/status`        | 返回 JSON，字段 `redis:"ok"` 视为 OK              |
| 版本信息     | GET | `/actuator/info` | 返回 JSON，读取 `name`/`version`/`build.time` 等 |
| 延迟历史     | —   | 本地存储             | 记录 `/healthy` 的历史延时（localStorage）          |
| 依赖（可选）   | GET | 自定义多端点           | 通过 `deps=name:/path,...` 配置                |
| 静态资源热身   | GET | 自定义多 URL         | 通过 `assets=/a.js,/b.css` 等传入               |
| 一键压测     | GET | 任意端点             | 浏览器内轻载压测，总量/并发/超时可配                        |

> 前端所有请求 `mode: same-origin`、`cache: no-store`，默认超时 8s（可在源码中修改）。

---

## 核心接口契约

### `/`（根路径）

- 用于检查外网 HTTP 可达性及跳转链路。返回 `2xx/3xx` 视为 OK。

### `/healthy`

- 用于后端就绪检查。返回 `200 OK` 即视为健康，响应体可为空或 JSON。
- 示例：

```http
HTTP/1.1 200 OK
Content-Type: text/plain

OK
```

或

```json
{
  "status": "UP"
}
```

### `/status`

- 用于聚合下游（如 Redis）状态。前端只识别 `redis` 字段：存在且（大小写不敏感）为 `"ok"` 则 OK。
- 建议返回：

```json
{
  "redis": "ok",
  "db": "ok",
  "uptimeSec": 123456,
  "details": {
    "redisPingMs": 2
  }
}
```

### `/actuator/info` 或 `/version`

- 前端会尝试从返回 JSON 中推断：名称、版本、时间，支持这些常见字段：
    - `build.version`、`git.commit.id`、`version`、`app.version`、`app.name`、`name`、`build.time`、`time`、`timestamp`
- Spring Boot Actuator 示例：

```json
{
  "build": {
    "version": "1.2.3",
    "time": "2025-08-22T06:12:31Z"
  },
  "app": {
    "name": "convertor"
  }
}
```

### 依赖面板（可选）

- 通过 Query 传入：`?deps=redis:/status,profile:/api/profile,orders:/api/orders/health`
- 每个依赖以 `name:/path` 形式，逗号分隔。前端仅根据 `HTTP 2xx` 判定 OK；如需更细规则，请让该依赖指向你的自定义健康端点。

### 静态资源热身（可选）

- 传入多个 URL，前端并发请求以预热 CDN/缓存，统计成功/失败。
- 通过 Query：`?assets=/assets/a.js,/assets/b.css,/assets/img/logo.png`，或在页面输入框填写。

---

## 前端可配置项（Query）

- `ping`：外网可达检测路径（默认 `/`）
- `health`：后端就绪路径（默认 `/healthy`）
- `redis`：聚合状态路径（默认 `/status`）
- `info`：版本信息路径（默认 `/actuator/info`）
- `deps`：依赖列表，格式 `name:/path,name2:/path2`
- `assets`：热身 URL 列表，格式 `/a.js,/b.css,/img.png`

示例：

```
/?health=/healthz&redis=/svc/status&info=/version&deps=redis:/svc/status,profile:/api/profile/health&assets=/a.js,/b.css
```

---

## 持久化与本地存储

- 延迟历史仅记录 `/healthy` 的往返延时，使用 `localStorage`。
- Key 规则：`gdp_latency_history::<origin>::<name>`，当前 `name` 为 `health`。
- 最多保存 500 条，页面加载时恢复，新的检测结果会 append 并重绘曲线。

---

## 一键压测说明

- 浏览器内发起，非严格基准，适合作为“轻载冒烟”。
- 参数：
    - `Total`：总请求数（默认 100）
    - `Concurrency`：并发数（默认 8）
    - `Timeout(ms)`：单请求超时（默认 8000）
- 输出：完成数、成功数、失败数、错误率、min/p50/p95/p99/max。
- 前端保存最近一次配置到 `localStorage`（Key：`gdp_bench_cfg::<origin>`）。

---

## 预留日志面板设计

目标：将该面板做成“轻量控制台”，可浏览最近日志、检索并实时追踪。

### API 设计

- **写入（可选）**  
  `POST /logs/ingest`
    - Content-Type: `application/x-ndjson`（每行一个 JSON 对象）或 `application/json`（数组）
    - 返回：`202 Accepted` 或 `200 OK`

- **最近日志列表**  
  `GET /logs/recent?limit=500&level=INFO&since=2025-08-22T00:00:00Z&query=traceId:abc123`
    - 返回 JSON 数组（倒序）

- **实时流（推荐 SSE）**  
  `GET /logs/stream`
    - `text/event-stream`，事件名 `log`，数据为单条日志 JSON
    - 支持 `Last-Event-ID` 恢复

- **搜索（可选）**  
  `GET /logs/search?query=...&from=...&to=...&level=...`

### 消息格式

```json
{
  "ts": "2025-08-22T06:23:11.123Z",
  "level": "INFO",
  "service": "convertor",
  "host": "pod-1",
  "traceId": "a1b2c3",
  "spanId": "d4e5f6",
  "message": "Processed request",
  "tags": [
    "http",
    "ingress"
  ],
  "fields": {
    "path": "/healthy",
    "status": 200,
    "latency_ms": 12
  }
}
```

字段建议：`ts`（UTC ISO-8601）必填；`level`（DEBUG/INFO/WARN/ERROR）；`service`/`host`；`traceId`/`spanId`；`message`；`tags`；`fields`（键值）。

### 后端实现建议（Kotlin + Spring Boot）

- **Controller：** 暴露上述接口
- **存储：** 简单可用 H2/PostgreSQL；或落地文件配合 tail；生产建议接 ELK/ClickHouse
- **SSE：** `SseEmitter` 或 Reactor `Flux`，按订阅推送实时日志
- **安全：** 见下节

极简草图：

```kotlin
@RestController
@RequestMapping("/logs")
class LogsController {

  data class LogEvent(
    val ts: Instant,
    val level: String,
    val service: String?,
    val host: String?,
    val traceId: String?,
    val spanId: String?,
    val message: String,
    val tags: List<String> = emptyList(),
    val fields: Map<String, Any?> = emptyMap()
  )

  private val sink = Sinks.many().multicast().onBackpressureBuffer<LogEvent>()
  private val store = Collections.synchronizedList(mutableListOf<LogEvent>())

  @PostMapping("/ingest", consumes=["application/x-ndjson","application/json"])
  fun ingest(@RequestBody body: String): ResponseEntity<Void> {
    val lines = if (body.trim().startsWith("[")) jacksonObjectMapper().readValue<List<LogEvent>>(body)
                else body.lineSequence().filter { it.isNotBlank() }.map { jacksonObjectMapper().readValue<LogEvent>(it) }.toList()
    lines.forEach { e -> store.add(e); sink.tryEmitNext(e) }
    return ResponseEntity.accepted().build()
  }

  @GetMapping("/recent")
  fun recent(@RequestParam(defaultValue = "200") limit:Int): List<LogEvent> =
    store.takeLast(limit).reversed()

  @GetMapping("/stream", produces=[MediaType.TEXT_EVENT_STREAM_VALUE])
  fun stream(): Flux<ServerSentEvent<LogEvent>> =
    sink.asFlux().map { ServerSentEvent.builder(it).event("log").build() }
}
```

### 安全与配额

- 写入接口必须鉴权（Bearer Token / mTLS）
- 读接口建议仅内网或受保护的管理端口
- 速率限制与请求体大小限制
- CORS 限制为你的前端域名
