import { ApiResponse } from "../../common/response/response";

/**
 * 测速状态：成功、超时、出错
 */
export enum ResponseStatus {
    OK = "OK",
    TIMEOUT = "TIMEOUT",
    ERROR = "ERROR",
}

/**
 * Resource Timing 的阶段拆解（单位：毫秒）
 */
export interface LatencyPhases {
    /** 重定向耗时 */
    redirectMs?: number;

    /** DNS 解析耗时 */
    dnsMs?: number;

    /** TCP 连接耗时（含三次握手，不含 TLS） */
    connectMs?: number;

    /** TLS 握手耗时 */
    tlsMs?: number;

    /** 从发起请求到收到响应首字节（TTFB） */
    requestMs?: number;

    /** 与 TTFB 等价（语义更直观） */
    ttfbMs?: number;

    /** 内容下载耗时（首字节到最后一字节） */
    contentDownloadMs?: number;

    /** 实际传输体积（字节） */
    transferSize?: number;

    /** 编码后主体大小（字节） */
    encodedBodySize?: number;

    /** 解码后主体大小（字节） */
    decodedBodySize?: number;
}

/**
 * 服务器 Server-Timing 头解析结果
 * 键为指标名，值为耗时（毫秒）
 */
export type ServerTimingMap = Record<string, number>;

/**
 * `LatencyService.fetchWithLatency` 的可选项
 */
export interface MeasureOptions extends RequestInit {
    /** 超时时间（毫秒），到时会 Abort */
    timeoutMs?: number;

    /** 用于注入 RT 关联 ID 的查询参数名，默认 "rtid" */
    rtidParam?: string;
}

/**
 * 单次请求的测速结果（时间戳均为 Epoch 毫秒）
 */
export interface LatencyResult<T = void> {
    /** 实际请求的最终 URL（包含 rtid） */
    url: string;

    /** HTTP 方法（大写） */
    method: string;

    /** 测速状态：OK/TIMEOUT/ERROR */
    status: ResponseStatus;

    /** HTTP 状态码（若出错/超时可能为 0） */
    httpStatus: number;

    /** HTTP 是否为 2xx */
    httpOk: boolean;

    /** 启动请求的时间点（Epoch ms） */
    startedAt: number;

    /** 收到响应头（TTFB）时间点（Epoch ms） */
    headersAt: number;

    /** 读取完整 JSON 结束时间点（Epoch ms） */
    endedAt: number;

    /** TTFB（毫秒），优先来自 Performance，其次来自自测 */
    ttfbMs: number;

    /** 总耗时（毫秒），优先来自 Performance（duration），其次自测 */
    totalMs: number;

    /** 服务器 Server-Timing 头汇总（毫秒） */
    serverTiming?: ServerTimingMap;

    /** Resource Timing 分阶段耗时（毫秒） */
    phases?: LatencyPhases;

    /** 业务响应体（已包装成 ApiResponse<T>） */
    response?: ApiResponse<T>;

    /** 出错信息（仅在 ERROR/TIMEOUT 时可能出现） */
    errorMessage?: string;

    /** 本次请求使用的 rtid 值（方便排查） */
    rtid: string;
}
