import { AfterViewInit, Component, DestroyRef, effect, ElementRef, inject, input } from "@angular/core";
import { BehaviorSubject, filter, interval, map, scan, Subscription, switchMap } from "rxjs";
import { EChartHandle, initEChart } from "../../../../common/echarts/echarts.factory";
import { ECOption } from "../../../../common/echarts/echarts.registry";
import { DashboardService } from "../../../../service/dashboard.service";
import { Hz, TickerService } from "../../../../service/ticker.service";

@Component({
    selector: "app-actuator-line-chart",
    imports: [],
    templateUrl: "./actuator-line-chart.html",
    styleUrl: "./actuator-line-chart.scss",
})
export class ActuatorLineChart implements AfterViewInit {
    // 注入
    protected readonly host = inject<ElementRef<HTMLElement>>(ElementRef<HTMLElement>);
    protected readonly destroyRef = inject(DestroyRef);
    protected readonly timeTickerService = inject(TickerService);
    protected readonly dashboardService = inject(DashboardService);

    // 参数
    public readonly windowMs = input<number>(10_000);
    public readonly shiftDelayMs = input<number>(1_500);
    public readonly targetFps = input<Hz>(144);

    // 内部状态
    protected chartHandle!: EChartHandle;
    protected pendingSources: LineChartSample[][] | null = null;

    public readonly valueTicker = interval(1000);
    public readonly next: BehaviorSubject<LineChartSample[]> = new BehaviorSubject([] as LineChartSample[]);
    public readonly next$ = this.next.pipe(
        filter(v => !!v && v.length > 0),
        map(v => v!),
    );
    protected nextSubscription: Subscription = this.next$.pipe(
        scan((acc: LineChartSample[][], value) => {
            while (acc.length < value.length) {
                acc.push([]);
            }
            for (let i = 0; i < acc.length; ++i) {
                acc[i].push(value[i]);
                if (acc[i].length > 13) {
                    acc[i].shift();
                }
            }
            return acc;
        }, []),
    ).subscribe(source => {
        this.pendingSources = source;
    });

    protected offValueTick = effect((onCleanup) => {
        const sub = this.valueTicker
            .pipe(
                switchMap(async () => {
                    const healthLatency = await this.dashboardService.healthLatency();
                    const healthSample: LineChartSample = {
                        time: healthLatency.startedAt,
                        value: healthLatency.totalMs,
                    };
                    const redisLatency = await this.dashboardService.redisLatency();
                    const redisSample: LineChartSample = { time: healthLatency.startedAt, value: redisLatency.totalMs };
                    return [ healthSample, redisSample ];
                }),
            )
            .subscribe((sample) => {
                this.next.next(sample);
            });
        onCleanup(() => {
            sub.unsubscribe();
        });
    });

    protected offTimeTick: (() => void) | null = null;

    ngAfterViewInit(): void {
        this.chartHandle = initEChart(this.host.nativeElement);
        this.chartHandle.chart.setOption(this.lineOption());

        this.toggle();

        this.destroyRef.onDestroy(() => {
            this.nextSubscription.unsubscribe();
            this.chartHandle.dispose();
            this.offValueTick.destroy();
            this.offTimeTick?.();
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
        if (this.pendingSources && this.pendingSources.length > 0) {
            options.dataset = [
                {
                    id: "healthy",
                    dimensions: [ "time", "value" ],
                    source: this.pendingSources[0],
                },
                {
                    id: "redis",
                    dimensions: [ "time", "value" ],
                    source: this.pendingSources[1],
                },
            ];
            this.pendingSources = null;
        }
        this.chartHandle?.chart.setOption(options);
    }


    // 折线图
    private lineOption(): ECOption {
        return {
            grid: { left: 0, right: 0, top: 2, bottom: 2 },
            animationDuration: 300,          // 首次渲染 300ms 内完成
            animationEasing: "linear",       // 线性更利落
            animationDelay: 0,
            animationDurationUpdate: 200,    // 后续更新也快一点
            animationEasingUpdate: "linear",
            dataset: [
                {
                    id: "healthy",
                    dimensions: [ "time", "value" ],
                    source: [],
                },
                {
                    id: "redis",
                    dimensions: [ "time", "value" ],
                    source: [],
                },
            ],
            // dataZoom: [
            //     {
            //         // id: 'dataZoomY',
            //         type: "inside",
            //         yAxisIndex: 0,
            //         filterMode: "none",
            //         startValue: 0,
            //         endValue: 10,
            //     },
            //     {
            //         // id: 'dataZoomY',
            //         type: "inside",
            //         yAxisIndex: 0,
            //         filterMode: "none",
            //         startValue: 240,
            //         endValue: 340,
            //     },
            // ],
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
            yAxis: [
                {
                    id: "healthy",
                    type: "value",
                    // axisLabel: undefined,
                    // axisLine: undefined,
                    // splitLine: undefined,
                    min: extent => Math.max(extent.min, -1),
                    max: extent => extent.max + 1,
                    scale: true,
                    splitNumber: 4,
                },
            ],
            series: [
                {
                    id: "healthy",
                    datasetId: "healthy",
                    type: "line",
                    showSymbol: false,
                    encode: { x: "time", y: "value" },
                    smooth: false,
                },
                {
                    id: "redis",
                    datasetId: "redis",
                    type: "line",
                    showSymbol: false,
                    encode: { x: "time", y: "value" },
                    smooth: false,
                },
            ],
        };
    }
}

export interface LineChartSample {
    time: number;

    value: number;
}
