import { echarts } from "./echarts.registry";

export interface EChartHandle {
    chart: ReturnType<typeof echarts.init>;

    /** 断开 ResizeObserver / 事件监听并 dispose 图表 */
    dispose(): void;
}

/** 直接可用的 init：自动监听容器尺寸变化并 chart.resize() */
export function initEChart(
    dom: HTMLElement,
    options?: {
        theme?: string;
        renderer?: "canvas" | "svg";
        /** rAF 合并频繁 resize，默认 true */
        coalesceWithRaf?: boolean;
        /** 如果没有 ResizeObserver，则退化到 window.resize，默认 true */
        fallbackToWindow?: boolean;
    },
): EChartHandle {
    const {
        theme,
        renderer = "canvas",
        coalesceWithRaf = true,
        fallbackToWindow = true,
    } = options ?? {};

    const chart = echarts.init(dom, theme, { renderer });

    let rafId: number | null = null;
    const scheduleResize = () => {
        if (!coalesceWithRaf) {
            chart.resize();
            return;
        }
        if (rafId == null) {
            rafId = requestAnimationFrame(() => {
                rafId = null;
                chart.resize();
            });
        }
    };

    // 优先用 ResizeObserver
    let ro: ResizeObserver | null = null;
    // 退化方案的引用，便于移除
    let onWinResize: ((this: Window, ev: UIEvent) => any) | null = null;

    if (typeof ResizeObserver !== "undefined") {
        ro = new ResizeObserver(() => scheduleResize());
        ro.observe(dom);
    } else if (fallbackToWindow) {
        onWinResize = () => scheduleResize();
        // window.resize 在某些布局场景下也能覆盖容器变更（例如百分比宽高）
        window.addEventListener("resize", onWinResize);
    }

    // 有些浏览器首次布局延后，下一帧再补一次
    requestAnimationFrame(() => chart.resize());

    const dispose = () => {
        if (ro) {
            ro.disconnect();
            ro = null;
        }
        if (onWinResize) {
            window.removeEventListener("resize", onWinResize);
            onWinResize = null;
        }
        if (rafId != null) {
            cancelAnimationFrame(rafId);
            rafId = null;
        }
        chart.dispose();
    };

    return { chart, dispose };
}

/** 原始 init：不做任何监听，给需要完全自管生命周期的场景 */
export function initEChartRaw(
    dom: HTMLElement,
    theme?: string,
    renderer: "canvas" | "svg" = "canvas",
) {
    return echarts.init(dom, theme, { renderer });
}
