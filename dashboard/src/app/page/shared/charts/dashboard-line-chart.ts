import { AfterViewInit, DestroyRef, Directive, ElementRef, inject, input } from "@angular/core";
import { BehaviorSubject, Observable, scan, Subscription } from "rxjs";
import { EChartHandle, initEChart } from "../../../common/echarts/echarts.factory";
import { ECOption } from "../../../common/echarts/echarts.registry";
import { DashboardService } from "../../../service/dashboard-service";
import { Hz, TickerService } from "../../../service/ticker-service";

@Directive()
export abstract class DashboardLineChart implements AfterViewInit {
    // 注入
    protected readonly host = inject<ElementRef<HTMLElement>>(ElementRef<HTMLElement>);
    protected readonly destroyRef = inject(DestroyRef);
    protected readonly timeTickerService = inject(TickerService);
    protected readonly dashboardService = inject(DashboardService);

    // 参数
    public readonly valueTicker = input.required<Observable<number>>();
    public readonly windowMs = input<number>(10_000);
    public readonly shiftDelayMs = input<number>(2_000);
    public readonly targetFps = input<Hz>(144);

    // 内部状态
    public readonly next: BehaviorSubject<LineChartSample> = new BehaviorSubject({ time: Date.now(), value: 0 });
    protected chartHandle!: EChartHandle;
    protected pendingSource: LineChartSample[] | null = [ { time: Date.now(), value: 0 } ];
    protected subscribeNext: Subscription | null = null;
    protected offTimeTick: (() => void) | null = null;

    ngAfterViewInit(): void {
        this.chartHandle = initEChart(this.host.nativeElement);
        this.chartHandle.chart.setOption(this.lineOption([]));

        this.subscribeNext = this.next.pipe(
            scan((acc: LineChartSample[], value) => {
                acc.push(value);
                if (acc.length > 13) {
                    acc.shift();
                }
                return acc;
            }, []),
        ).subscribe(source => {
            this.pendingSource = source;
        });

        this.toggle();

        this.destroyRef.onDestroy(() => {
            this.subscribeNext?.unsubscribe();
            this.chartHandle.dispose();
        });
    }

    toggle(): void {
        if (this.offTimeTick) {
            this.offTimeTick();
            this.offTimeTick = null;
        } else {
            this.offTimeTick = this.timeTickerService.onTick(this.onTick.bind(this));
        }
    }

    onTick(nowMs: number, dtMs: number) {
        const now = Date.now();
        const windowMs = this.windowMs();
        const shiftDelayMs = this.shiftDelayMs();
        const min = now - windowMs - shiftDelayMs;
        const max = now - shiftDelayMs;
        let options: ECOption = {
            xAxis: {
                min,
                max,
            },
        };
        if (this.pendingSource && this.pendingSource.length > 0) {
            options.series = [ { id: "latency" } ];
            options.dataset = {
                source: this.pendingSource,
            };
            this.pendingSource = null;
        }
        this.chartHandle?.chart.setOption(options);
    }


    // 折线图
    lineOption(
        data: LineChartSample[],
    ): ECOption {
        return {
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
}

export interface LineChartSample {
    time: number;

    value: number;
}
