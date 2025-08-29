// ------------------------------------
// latency-utils.ts
// ------------------------------------
import { PhaseBreakdown, ServerTimingMap } from "./latency-types";

/**
 * 将 rtid 作为查询参数附加到 URL 上（保持原有参数）
 */
export function addRtidToUrl(input: RequestInfo | URL, rtidParam: string, rtid: string): string {
    const base = typeof input === "string"
        ? input
        : (input as Request).url ?? (input as URL).toString?.() ?? String(input);

    const u = new URL(base, location.href);
    u.searchParams.set(rtidParam, rtid);
    return u.toString();
}

/**
 * 创建带超时的 AbortController
 */
export function createAbortControllerWithTimeout(
    timeoutMs?: number,
    externalSignal?: AbortSignal | null,
): { controller: AbortController; cancelTimer: () => void; didTimeout: () => boolean } {
    const controller = new AbortController();
    let timedOut = false;

    // 透传外部 signal
    if (externalSignal) {
        if (externalSignal.aborted) {
            controller.abort();
        } else {
            externalSignal.addEventListener("abort", () => controller.abort(), { once: true });
        }
    }

    let timer: number | undefined;
    if (typeof timeoutMs === "number" && timeoutMs > 0) {
        timer = setTimeout(() => {
            timedOut = true;
            controller.abort();
        }, timeoutMs);
    }

    const cancelTimer = () => {
        if (timer !== undefined) {
            clearTimeout(timer);
        }
    };

    const didTimeout = () => timedOut;

    return { controller, cancelTimer, didTimeout };
}

/**
 * 通过完整 URL 精确获取 PerformanceResourceTiming（rtid 确保唯一）
 */
export function getResourceTimingByExactUrl(url: string): PerformanceResourceTiming | undefined {
    const list = performance.getEntriesByName(url, "resource") as PerformanceResourceTiming[];
    if (!list || list.length === 0) return undefined;
    // rtid 唯一的情况下，理论上只会有一条，稳妥起见取最后一条
    return list[list.length - 1];
}

/**
 * 构建阶段分解（需要 TAO，否则部分字段为 0）
 */
export function buildPhaseBreakdown(e: PerformanceResourceTiming): PhaseBreakdown {
    const ms = (x: number) => (Number.isFinite(x) ? x : 0);
    const between = (a: number, b: number) => (a && b ? Math.max(0, b - a) : 0);

    const redirectMs = between(ms(e.redirectStart), ms(e.redirectEnd));
    const dnsMs = between(ms(e.domainLookupStart), ms(e.domainLookupEnd));
    const connectMs = between(ms(e.connectStart), ms(e.connectEnd));
    const tlsMs = e.secureConnectionStart ? between(ms(e.secureConnectionStart), ms(e.connectEnd)) : 0;
    const requestMs = between(ms(e.requestStart), ms(e.responseStart));
    const ttfbMs = requestMs;
    const contentDownloadMs = between(ms(e.responseStart), ms(e.responseEnd));

    return {
        redirectMs: round(redirectMs),
        dnsMs: round(dnsMs),
        connectMs: round(connectMs),
        tlsMs: round(tlsMs),
        requestMs: round(requestMs),
        ttfbMs: round(ttfbMs),
        contentDownloadMs: round(contentDownloadMs),
        transferSize: e.transferSize || undefined,
        encodedBodySize: e.encodedBodySize || undefined,
        decodedBodySize: e.decodedBodySize || undefined,
    };
}

/**
 * 解析 Server-Timing 响应头
 */
export function parseServerTiming(header: string): ServerTimingMap {
    const out: ServerTimingMap = {};
    for (const raw of header.split(",")) {
        const token = raw.trim();
        const name = token.split(";")[0]?.trim();
        const m = token.match(/dur=([\d.]+)/);
        if (name && m) out[name] = Number(m[1]);
    }
    return out;
}

/**
 * 下一帧（确保 RT entry 已写入）
 */
export function nextFrame(): Promise<void> {
    return new Promise((resolve) => {
        if (typeof requestAnimationFrame !== "undefined") {
            // rAF 的回调签名是 (ts: DOMHighResTimeStamp) => void
            requestAnimationFrame(() => resolve());
        } else {
            // 极少数环境的兜底
            setTimeout(() => resolve(), 0);
        }
    });
}

/**
 * 四舍五入为非负整数
 */
export function round(n: number): number {
    return Math.max(0, Math.round(n));
}

/**
 * 类型守卫：是否 JSON Content-Type
 */
export function isJsonContentType(contentType: string | null): contentType is string {
    return !!contentType && /(^|\s|;)application\/json/i.test(contentType);
}

/**
 * 类型守卫：是否形如 ApiResponse<T> 的对象
 * 注意：只是结构性判断，不保证完全正确
 */
export function isApiResponseLike(x: unknown): x is { status: number; message?: string; data?: unknown } {
    if (typeof x !== "object" || x === null) return false;
    const anyX = x as Record<string, unknown>;
    return typeof anyX["status"] === "number";
}

// latency-utils.ts
export function hrToEpochMs(relativeMs: number): number {
    return performance.timeOrigin + relativeMs;
}
