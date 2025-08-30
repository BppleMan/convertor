import { DataZoomComponentOption, VisualMapComponentOption } from "echarts";
import type { BarSeriesOption, LineSeriesOption, PieSeriesOption } from "echarts/charts";
// 图表
import { BarChart, LineChart, PieChart } from "echarts/charts";

// 组件
import {
    DatasetComponent,
    DatasetComponentOption,
    DataZoomComponent,
    GridComponent,
    GridComponentOption,
    LegendComponent,
    LegendComponentOption,
    TitleComponent,
    TitleComponentOption,
    TooltipComponent,
    TooltipComponentOption,
    TransformComponent,
    VisualMapComponent,
} from "echarts/components";
import type { ComposeOption } from "echarts/core";
import * as echarts from "echarts/core";

// 特性与渲染器
import { LabelLayout, UniversalTransition } from "echarts/features";
import { CanvasRenderer } from "echarts/renderers";

// —— 组合“最小” Option 类型 ——
// 你只会用到的：折线、柱状、饼图 + 常用组件
export type ECOption = ComposeOption<
    | LineSeriesOption
    | BarSeriesOption
    | PieSeriesOption
    | GridComponentOption
    | TitleComponentOption
    | TooltipComponentOption
    | LegendComponentOption
    | DatasetComponentOption
    | VisualMapComponentOption
    | DataZoomComponentOption
>;

// —— 注册必须的图表与组件 ——
// 后续如果加了 DataZoom / VisualMap 等，记得这里和上面的类型都要加
echarts.use([
    LineChart,
    BarChart,
    PieChart,

    GridComponent,
    TitleComponent,
    TooltipComponent,
    LegendComponent,
    DatasetComponent,
    TransformComponent,
    VisualMapComponent,
    DataZoomComponent,

    LabelLayout,
    UniversalTransition,
    CanvasRenderer,
]);

// 导出 echarts 实例工厂（可选：统一用 Canvas 渲染器）
export { echarts };
