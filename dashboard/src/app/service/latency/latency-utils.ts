/**
 * 将 DOMHighResTimeStamp（performance 基准）转为 Epoch 毫秒
 */
export function toEpochMs(highResMs: number): number {
    // performance.timeOrigin 是 Epoch 毫秒（小数），相加后得到 Epoch 毫秒
    return Math.round(performance.timeOrigin + highResMs);
}

/**
 * 取当前时间（Epoch 毫秒），优先使用 performance
 */
export function nowEpochMs(): number {
    return toEpochMs(performance.now());
}

/**
 * 下一帧（rAF），用于等待 PerformanceResourceTiming 入队
 */
export function nextFrame(): Promise<void> {
    return new Promise((resolve) => {
        if (typeof requestAnimationFrame === "function") {
            requestAnimationFrame(() => resolve());
        } else {
            // 兜底：与 rAF 不同，setTimeout 需要指定毫秒
            setTimeout(() => resolve(), 0);
        }
    });
}

/**
 * 将给定 URL 注入 rtid 查询参数，返回字符串形式的最终 URL
 */
export function addRtid(url: string | URL, rtid: string, paramName: string): string {
    const u = typeof url === "string" ? new URL(url, location.href) : new URL(url.toString());
    u.searchParams.set(paramName, rtid);
    return u.toString();
}

/**
 * 解析 Server-Timing 响应头为 Map<name, durMs>
 */
export function parseServerTimingHeader(value: string | null): Record<string, number> | undefined {
    if (!value) return undefined;
    const out: Record<string, number> = {};
    for (const part of value.split(",")) {
        const token = part.trim();
        if (!token) continue;
        const name = token.split(";")[0]?.trim();
        const durMatch = token.match(/dur=([\d.]+)/i);
        if (name && durMatch) {
            const n = Number(durMatch[1]);
            if (Number.isFinite(n)) out[name] = n;
        }
    }
    return Object.keys(out).length ? out : undefined;
}

/**
 * 是否为 AbortError
 */
export function isAbortError(err: unknown): err is DOMException {
    return !!(err && typeof err === "object" && (err as any).name === "AbortError");
}

/**
 * 是否为 PerformanceResourceTiming
 */
export function isPerfResourceTiming(x: unknown): x is PerformanceResourceTiming {
    return !!(x && typeof x === "object" && "entryType" in (x as any) && (x as any).entryType === "resource");
}

/**
 * 精确获取与 URL 完全匹配的 PerformanceResourceTiming
 * 由于使用 rtid，理论上只会命中唯一条目
 */
export function getExactResourceTimingByName(url: string): PerformanceResourceTiming | undefined {
    const list = performance.getEntriesByName(url, "resource") as PerformanceResourceTiming[];
    if (!Array.isArray(list) || list.length === 0) return undefined;
    // 优先选择 initiatorType 为 fetch/xhr 的项
    const hit = list.find(e => (e.initiatorType === "fetch" || e.initiatorType === "xmlhttprequest"));
    return hit ?? list[0];
}

/**
 * 从 PerformanceResourceTiming 构建阶段耗时
 */
export function buildPhases(e: PerformanceResourceTiming) {
    const between = (a?: number, b?: number) =>
        Number.isFinite(a) && Number.isFinite(b) && a! > 0 && b! > 0 ? Math.max(0, Math.round(b! - a!)) : 0;

    const redirectMs = between(e.redirectStart, e.redirectEnd);
    const dnsMs = between(e.domainLookupStart, e.domainLookupEnd);
    const connectMs = between(e.connectStart, e.connectEnd);
    const tlsMs = Number.isFinite(e.secureConnectionStart) && e.secureConnectionStart! > 0
        ? between(e.secureConnectionStart, e.connectEnd)
        : 0;
    const requestMs = between(e.requestStart, e.responseStart);
    const contentDownloadMs = between(e.responseStart, e.responseEnd);

    return {
        redirectMs: redirectMs || undefined,
        dnsMs: dnsMs || undefined,
        connectMs: connectMs || undefined,
        tlsMs: tlsMs || undefined,
        requestMs: requestMs || undefined,
        ttfbMs: requestMs || undefined,
        contentDownloadMs: contentDownloadMs || undefined,
        transferSize: e.transferSize || undefined,
        encodedBodySize: e.encodedBodySize || undefined,
        decodedBodySize: e.decodedBodySize || undefined,
    };
}
