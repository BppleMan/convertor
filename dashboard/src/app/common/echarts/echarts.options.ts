import type { ECOption } from "./echarts.registry";


// 饼图
export function pieOption(
    items: Array<{ name: string; value: number }>,
    title?: string,
): ECOption {
    return {
        title: title ? { text: title, left: "center" } : undefined,
        tooltip: { trigger: "item" },
        legend: { bottom: 0 },
        series: [
            {
                type: "pie",
                radius: "60%",
                center: [ "50%", "45%" ],
                data: items,
                // roseType: 'radius', // 需要玫瑰图时打开
            },
        ],
    };
}
