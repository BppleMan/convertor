// ------------------------------------
// latency-types.ts
// ------------------------------------

import { ApiResponse } from "../../common/response/response";

/**
 * 响应体读取模式（用 enum 而非 union type）
 */
export enum ReadBodyMode {
    /** 不读取响应体，仅测 TTFB */
    None = "None",
    /** 读取并丢弃字节，只统计下载大小与总耗时 */
    Drain = "Drain",
    /** 将响应体解析为 JSON */
    Json = "Json",
    /** 将响应体解析为文本 */
    Text = "Text",
    /** 将响应体解析为 ArrayBuffer */
    ArrayBuffer = "ArrayBuffer",
    /** 将响应体解析为 Blob */
    Blob = "Blob",
}

/**
 * 请求状态（ok / timeout / error）
 */
export enum LatencyState {
    /** 请求成功（可能 HTTP 非 2xx 也会在状态里标记为 Error，详见 service 逻辑） */
    Ok = "Ok",
    /** 因超时中止 */
    Timeout = "Timeout",
    /** 其它错误（网络/HTTP 非 ok/解析错误等） */
    Error = "Error",
}

/**
 * Resource Timing 阶段分解
 */
export interface PhaseBreakdown {
    /** 重定向耗时（毫秒） */
    redirectMs?: number;

    /** DNS 解析耗时（毫秒） */
    dnsMs?: number;

    /** TCP 连接耗时（毫秒） */
    connectMs?: number;

    /** TLS 握手耗时（毫秒） */
    tlsMs?: number;

    /** 请求发送至首字节（TTFB 前一段）（毫秒） */
    requestMs?: number;

    /** 首字节时间（毫秒），与 headersLatencyMs 对齐 */
    ttfbMs?: number;

    /** 内容下载耗时（毫秒） */
    contentDownloadMs?: number;

    /** 传输总大小（字节，可能含头部） */
    transferSize?: number;

    /** 压缩后主体大小（字节） */
    encodedBodySize?: number;

    /** 解压后主体大小（字节） */
    decodedBodySize?: number;
}

/**
 * Server-Timing 指标映射
 */
export type ServerTimingMap = Record<string, number>;

/**
 * fetch 的测量选项（扩展 RequestInit）
 */
export interface FetchLatencyOptions extends RequestInit {
    /** 请求超时（毫秒），到期触发 Abort；默认 5_000 */
    timeoutMs?: number;

    /** 响应体读取模式；默认 ReadBodyMode.Json */
    readBody?: ReadBodyMode;

    /** 是否启用 Resource Timing（默认 true） */
    useResourceTiming?: boolean;

    /** 附加在 URL 上的 rtid 参数名（默认 "rtid"） */
    rtidParam?: string;

    /** 是否自动向 URL 附加 rtid（默认 true） */
    appendRtid?: boolean;
}

/**
 * 统一的延迟测量结果
 */
export interface LatencyResult<T = void> {
    /** 请求的最终 URL（包含 rtid） */
    url: string;

    /** HTTP 方法（大写） */
    method: string;

    /** 请求状态（ok/timeout/error） */
    state: LatencyState;

    /** HTTP 状态码；网络错误/超时等为 0 */
    status: number;

    /** 是否 HTTP ok（2xx-3xx）；仅作参考 */
    ok: boolean;

    /** rtid 唯一标识，用于性能条目匹配 */
    rtid: string;

    /** 请求开始时间（performance 时间基准） */
    startedAtMs: number;

    /** 首字节时间点（收到响应头的时间） */
    headersAtMs: number;

    /** 结束时间点（按读取策略与可用的 RT 决定） */
    endedAtMs: number;

    /** TTFB（毫秒） */
    headersLatencyMs: number;

    /** 总耗时（毫秒） */
    totalLatencyMs: number;

    /** 估算的响应体大小（字节） */
    sizeBytes?: number;

    /** 服务器的 Server-Timing 指标 */
    serverTiming?: ServerTimingMap;

    /** Resource Timing 分解阶段（需要 TAO） */
    phases?: PhaseBreakdown;

    /** 解析后的响应体（根据读取模式） */
    value?: T | ApiResponse<T>;

    /** 错误信息（仅当 Error/Timeout 时可能存在） */
    error?: string;

    /** 是否因 AbortController 中止 */
    aborted?: boolean;

    /** 可选：原始 Response（调试用） */
    response?: Response;

    /** 请求开始的绝对时间（Unix ms） */
    startedAtEpochMs: number;

    /** 首字节时间点的绝对时间（Unix ms） */
    headersAtEpochMs: number;

    /** 结束时间点的绝对时间（Unix ms） */
    endedAtEpochMs: number;
}

