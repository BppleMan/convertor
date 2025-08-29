import type { ECOption } from "./echarts.registry";

export interface SampleData {
    time: number;
    value: number;
}

// 折线图
export function lineOption(
    data: SampleData[],
    title?: string,
): ECOption {
    return {
        title: title ? { text: title } : undefined,
        grid: {
            left: 0,
            right: 0,
            top: 2,
            bottom: 2,
        },
        animationDuration: 300,          // 首次渲染 300ms 内完成
        animationEasing: "linear",       // 线性更利落
        animationDelay: 0,
        animationDurationUpdate: 200,    // 后续更新也快一点
        animationEasingUpdate: "linear",
        // visualMap: [
        //     {
        //         show: false,
        //         type: "continuous",
        //         seriesIndex: 0,
        //         min: 0,
        //         max: 1000,
        //     },
        // ],
        dataset: {
            dimensions: [ "time", "value" ],
            source: data,
        },
        tooltip: {
            trigger: "axis",
            axisPointer: {
                animation: false,
            },
        },
        xAxis: {
            type: "time",
            axisLabel: undefined,
            axisLine: undefined,
            splitLine: undefined,
        },
        yAxis: {
            type: "value",
            // axisLabel: undefined,
            // axisLine: undefined,
            // splitLine: undefined,
            min: extent => Math.max(extent.min, -1),
            max: extent => extent.max + 1,
            splitNumber: 2,
        },
        series: [
            {
                id: "latency",
                type: "line",
                showSymbol: false,
                encode: { x: "time", y: "value" },
                smooth: true,
                progressive: 0,
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
