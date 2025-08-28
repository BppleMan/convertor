import type { ECOption } from "./echarts.registry";

// 折线图
export function lineOption(
    data: Array<[ number | string | Date, number ]>,
    title?: string,
): ECOption {
    return {
        title: title ? { text: title } : undefined,
        grid: { left: 40, right: 10, top: 20, bottom: 30 },
        tooltip: { trigger: "axis" },
        xAxis: { type: "time" },             // 时间轴（若用类目，改为 'category' 并给 data）
        yAxis: { type: "value", scale: true },
        series: [
            {
                type: "line",
                showSymbol: false,
                data, // [[timestamp, value], ...]
                // smooth: true, // 如需平滑
            },
        ],

    };
}

// 柱状图
export function barOption(
    categories: string[],
    values: number[],
    title?: string,
): ECOption {
    return {
        title: title ? { text: title } : undefined,
        grid: { left: 40, right: 10, top: 20, bottom: 40 },
        tooltip: { trigger: "axis" },
        xAxis: { type: "category", data: categories },
        yAxis: { type: "value" },
        series: [
            {
                type: "bar",
                data: values,
            },
        ],
        legend: undefined,
    };
}

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
