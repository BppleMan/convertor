import { Injectable } from "@angular/core";
import { ApiResponse } from "../../common/response/response";
// 按你的项目路径导入 ApiResponse（这里示例性指向 ./api-response）
import { LatencyResult, LatencyStatus, MeasureOptions } from "./latency-types";
import { addRtid, buildPhases, getExactResourceTimingByName, isAbortError, nextFrame, nowEpochMs, parseServerTimingHeader, toEpochMs } from "./latency-utils";

@Injectable({ providedIn: "root" })
export class LatencyService {
    /**
     * 发起 JSON 请求并测量延迟，自动注入 rtid，用 Performance 覆盖自测
     */
    public async fetchWithLatency<T = void>(
        input: string | URL,
        options: MeasureOptions = {},
    ): Promise<LatencyResult<T>> {
        const method = (options.method ?? "GET").toUpperCase();
        const rtid = crypto.randomUUID();
        const rtidParam = options.rtidParam ?? "rtid";

        const url = addRtid(input, rtid, rtidParam);

        // 构建 Abort + 超时
        const ac = new AbortController();
        const userSignal = options.signal;
        if (userSignal?.aborted) ac.abort();
        else if (userSignal) userSignal.addEventListener("abort", () => ac.abort(), { once: true });

        let timeoutId: number | undefined;
        if (typeof options.timeoutMs === "number" && options.timeoutMs > 0) {
            timeoutId = window.setTimeout(() => ac.abort(), options.timeoutMs);
        }

        // ===== 1) 自测：记录关键时间点（Epoch ms）
        let startedAt = nowEpochMs();
        let headersAt = startedAt;
        let endedAt = startedAt;
        let httpStatus = 0;
        let httpOk = false;
        let serverTiming: Record<string, number> | undefined;
        let responseObj: ApiResponse<T> | undefined;
        let status: LatencyStatus = LatencyStatus.ERROR;
        let errorMessage: string | undefined;

        try {
            const resp = await fetch(url, {
                ...options,
                signal: ac.signal,
            });

            headersAt = nowEpochMs();
            httpStatus = resp.status;
            httpOk = resp.ok;

            serverTiming = parseServerTimingHeader(resp.headers.get("server-timing"));

            // 一律按 JSON 解析并包装为 ApiResponse<T>
            const raw = await resp.json();
            responseObj = ApiResponse.deserialize<T>(raw);

            endedAt = nowEpochMs();

            // 先用自测结果给出粗略值
            let ttfbMs = Math.max(0, headersAt - startedAt);
            let totalMs = Math.max(0, endedAt - startedAt);

            // ===== 2) 尝试用 PerformanceResourceTiming 覆盖（rtid 唯一匹配）
            await nextFrame(); // 等待 RT 入队
            const entry = getExactResourceTimingByName(url);
            if (entry) {
                // 所有时间转换为 Epoch 毫秒
                const pStart = toEpochMs(entry.startTime);
                const pHeaders = toEpochMs(entry.responseStart);
                const pEnd = toEpochMs(entry.responseEnd);

                startedAt = pStart;
                headersAt = pHeaders;
                endedAt = pEnd;

                // duration 更精准（浏览器内部更靠近真实区间）
                totalMs = Math.max(0, Math.round(entry.duration));
                // TTFB 用 responseStart - requestStart
                if (Number.isFinite(entry.requestStart) && Number.isFinite(entry.responseStart)) {
                    ttfbMs = Math.max(0, Math.round(entry.responseStart - entry.requestStart));
                }

                const phases = buildPhases(entry);
                return {
                    url,
                    method,
                    status: LatencyStatus.OK,
                    httpStatus,
                    httpOk,
                    startedAt,
                    headersAt,
                    endedAt,
                    ttfbMs,
                    totalMs,
                    serverTiming,
                    phases,
                    response: responseObj,
                    rtid,
                };
            }

            // 没拿到 Performance：回退到自测
            status = LatencyStatus.OK;
            return {
                url,
                method,
                status,
                httpStatus,
                httpOk,
                startedAt,
                headersAt,
                endedAt,
                ttfbMs,
                totalMs,
                serverTiming,
                response: responseObj,
                rtid,
            };
        } catch (err: unknown) {
            endedAt = nowEpochMs();

            if (isAbortError(err)) {
                status = LatencyStatus.TIMEOUT;
                errorMessage = "Request timed out or aborted";
            } else {
                status = LatencyStatus.ERROR;
                errorMessage = (err as any)?.message ?? String(err);
            }

            // 即便出错/超时，也尝试用 Performance 修正起止时间（如果记录到了）
            try {
                await nextFrame();
                const entry = getExactResourceTimingByName(url);
                if (entry) {
                    const pStart = toEpochMs(entry.startTime);
                    const pHeaders = toEpochMs(entry.responseStart);
                    const pEnd = toEpochMs(entry.responseEnd);
                    startedAt = pStart;
                    // 若失败时 headers 未必达到，用 Performance 的字段更可信
                    headersAt = Number.isFinite(entry.responseStart) ? pHeaders : startedAt;
                    endedAt = Number.isFinite(entry.responseEnd) ? pEnd : endedAt;
                }
            } catch {
                // 忽略二次错误
            }

            const ttfbMs = Math.max(0, headersAt - startedAt);
            const totalMs = Math.max(0, endedAt - startedAt);

            return {
                url,
                method,
                status,
                httpStatus,
                httpOk,
                startedAt,
                headersAt,
                endedAt,
                ttfbMs,
                totalMs,
                serverTiming,
                response: responseObj,
                errorMessage,
                rtid,
            };
        } finally {
            if (timeoutId !== undefined) {
                clearTimeout(timeoutId);
            }
        }
    }
}
