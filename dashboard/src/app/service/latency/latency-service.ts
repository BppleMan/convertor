import { Injectable } from "@angular/core";
import { ApiResponse } from "../../common/response/response";
import { FetchLatencyOptions, LatencyResult, LatencyState, ReadBodyMode } from "./latency-types";
import { addRtidToUrl, buildPhaseBreakdown, createAbortControllerWithTimeout, getResourceTimingByExactUrl, hrToEpochMs, isApiResponseLike, isJsonContentType, nextFrame, parseServerTiming, round } from "./latency-utils";

/**
 * LatencyService：使用原生 fetch + Resource Timing 进行延迟测量（rtid 精确匹配）
 */
@Injectable({
    providedIn: "root",
})
export class LatencyService {
    /** rtid 参数名默认值 */
    private static readonly DEFAULT_RTID_PARAM = "rtid";

    /**
     * 发送请求并测量延迟（优先使用 Performance Resource Timing）
     */
    public async fetchWithLatency<T = unknown>(
        input: RequestInfo | URL,
        options: FetchLatencyOptions = {},
    ): Promise<LatencyResult<T>> {
        const method = (options.method ?? "GET").toString().toUpperCase();

        // 1) 生成 rtid 并构造最终 URL
        const rtidParam = options.rtidParam ?? LatencyService.DEFAULT_RTID_PARAM;
        const rtid = crypto.randomUUID();
        const url = options.appendRtid === false ? this.toUrlString(input) : addRtidToUrl(input, rtidParam, rtid);

        // 2) 超时控制器
        const { controller, cancelTimer, didTimeout } = createAbortControllerWithTimeout(
            options.timeoutMs ?? 5_000,
            options.signal,
        );

        // 3) 启动本地计时（回退用）；优先期望用 RT 的时间轴
        const perfNow = () => performance.now();
        let startedAtLocal = perfNow();
        let headersAtLocal = startedAtLocal;
        let endedAtLocal = startedAtLocal;

        // 4) 组装基础结果
        const base: LatencyResult<T> = {
            url,
            method,
            state: LatencyState.Error,
            status: 0,
            ok: false,
            rtid,

            startedAtMs: startedAtLocal,
            headersAtMs: headersAtLocal,
            endedAtMs: endedAtLocal,
            startedAtEpochMs: startedAtLocal,
            headersAtEpochMs: headersAtLocal,
            endedAtEpochMs: endedAtLocal,

            headersLatencyMs: 0,
            totalLatencyMs: 0,
        };

        try {
            // 5) 发起请求
            const resp = await fetch(url, { ...options, signal: controller.signal });
            headersAtLocal = perfNow();

            base.response = resp;
            base.status = resp.status;
            base.ok = resp.ok;

            // Server-Timing
            const stHeader = resp.headers.get("server-timing");
            if (stHeader) base.serverTiming = parseServerTiming(stHeader);

            // 6) 等待一帧，读取 RT（rtid 确保唯一）
            let entry = undefined as PerformanceResourceTiming | undefined;
            if (options.useResourceTiming !== false) {
                await nextFrame();
                entry = getResourceTimingByExactUrl(url);
                console.log(entry);
            }

            // 7) 读取响应体（根据模式）
            const readMode = options.readBody ?? ReadBodyMode.Json;

            let parsedValue: unknown = undefined;
            let sizeBytes: number | undefined = undefined;

            switch (readMode) {
                case ReadBodyMode.None: {
                    // 不读取 body，释放流
                    try {
                        await resp.body?.cancel();
                    } catch { /* noop */
                    }
                    break;
                }
                case ReadBodyMode.Drain: {
                    // 流式把字节读掉，不保留内容
                    if (resp.body) {
                        const reader = resp.body.getReader();
                        sizeBytes = 0;
                        while (true) {
                            const { done, value } = await reader.read();
                            if (done) break;
                            sizeBytes += value?.byteLength ?? 0;
                        }
                    } else {
                        const cl = resp.headers.get("content-length");
                        if (cl) sizeBytes = Number(cl);
                        const buf = await resp.arrayBuffer().catch(() => undefined);
                        if (buf && sizeBytes === undefined) sizeBytes = buf.byteLength;
                    }
                    break;
                }
                case ReadBodyMode.Json: {
                    parsedValue = await resp.json();
                    // 如果像 ApiResponse 结构，就包装起来
                    if (isApiResponseLike(parsedValue)) {
                        parsedValue = ApiResponse.deserialize(parsedValue);
                    }
                    // 计算体积（尽力）
                    const s = JSON.stringify(parsedValue);
                    sizeBytes = new TextEncoder().encode(s).byteLength;
                    break;
                }
                case ReadBodyMode.Text: {
                    const text = await resp.text();
                    parsedValue = text;
                    sizeBytes = new TextEncoder().encode(text).byteLength;
                    break;
                }
                case ReadBodyMode.ArrayBuffer: {
                    const buf = await resp.arrayBuffer();
                    parsedValue = buf;
                    sizeBytes = buf.byteLength;
                    break;
                }
                case ReadBodyMode.Blob: {
                    const blob = await resp.blob();
                    parsedValue = blob;
                    sizeBytes = blob.size;
                    break;
                }
                default: {
                    // 枚举防御式：理论不会到这里
                    break;
                }
            }

            // 如果没按 Json 模式，但 Content-Type 显示是 JSON，也帮你包装成 ApiResponse<T>
            // （满足“如果返回 content-type 是 json，直接包装”的诉求，同时保守判断结构）
            if (parsedValue === undefined && isJsonContentType(resp.headers.get("content-type"))) {
                try {
                    const val = await resp.clone().json();
                    parsedValue = isApiResponseLike(val) ? ApiResponse.deserialize<T>(val) : (val as unknown as T);
                    const s = JSON.stringify(val);
                    sizeBytes ??= new TextEncoder().encode(s).byteLength;
                } catch {
                    // JSON 解析失败则忽略自动包装
                }
            }

            // 8) 结束时间（本地）
            endedAtLocal = perfNow();

            // 9) 优先使用 RT 计算时间轴（rtid 精准匹配）
            if (entry) {
                const start = entry.startTime;
                const headersAt = entry.responseStart ? start + entry.responseStart : headersAtLocal;
                // 下载结束（若读取模式为 None，仅以 headers 为结束；否则以网络下载完成为结束）
                const end =
                    (readMode === ReadBodyMode.None)
                        ? headersAt
                        : (entry.responseEnd ? start + entry.responseEnd : endedAtLocal);

                base.startedAtMs = start;
                base.headersAtMs = headersAt;
                base.endedAtMs = end;

                base.headersLatencyMs = round(headersAt - start);
                base.totalLatencyMs = round(end - start);

                base.phases = buildPhaseBreakdown(entry);

                // 优先用 RT 提供的大小
                if (sizeBytes === undefined) {
                    sizeBytes = entry.decodedBodySize || entry.encodedBodySize || entry.transferSize || undefined;
                }
            } else {
                // 回退：使用本地时间（在你的环境里通常不会走到）
                base.startedAtMs = startedAtLocal;
                base.headersAtMs = headersAtLocal;
                base.endedAtMs = endedAtLocal;
                base.headersLatencyMs = round(headersAtLocal - startedAtLocal);
                base.totalLatencyMs = round(endedAtLocal - startedAtLocal);
            }

            base.sizeBytes = sizeBytes;
            base.value = parsedValue as T;

            // 10) 状态判定（enum）
            base.state = this.toState(didTimeout(), resp.ok);

            // HTTP 非 ok 也视为错误（但不是 Timeout）
            if (!resp.ok && base.state === LatencyState.Ok) {
                base.state = LatencyState.Error;
            }

            base.startedAtEpochMs = hrToEpochMs(base.startedAtMs);
            base.headersAtEpochMs = hrToEpochMs(base.headersAtMs);
            base.endedAtEpochMs = hrToEpochMs(base.endedAtMs);

            return base;
        } catch (err: unknown) {
            // 失败路径：网络/解析/Abort 等
            base.error = String((err as any)?.message ?? err);
            base.aborted = controller.signal.aborted;
            base.status = 0;
            base.ok = false;

            const end = perfNow();
            base.endedAtMs = end;
            base.totalLatencyMs = round(end - base.startedAtMs);
            base.headersLatencyMs = base.headersLatencyMs || 0;

            base.state = didTimeout() ? LatencyState.Timeout : LatencyState.Error;

            base.startedAtEpochMs = hrToEpochMs(base.startedAtMs);
            base.headersAtEpochMs = hrToEpochMs(base.headersAtMs);
            base.endedAtEpochMs = hrToEpochMs(base.endedAtMs);

            return base;
        } finally {
            cancelTimer();
        }
    }

    /**
     * 将输入转为 URL 字符串
     */
    private toUrlString(input: RequestInfo | URL): string {
        return typeof input === "string"
            ? input
            : (input as Request).url ?? (input as URL).toString?.() ?? String(input);
    }

    /**
     * 将 ok/timeout 映射到枚举状态
     */
    private toState(timedOut: boolean, httpOk: boolean): LatencyState {
        if (timedOut) return LatencyState.Timeout;
        return httpOk ? LatencyState.Ok : LatencyState.Error;
    }
}
