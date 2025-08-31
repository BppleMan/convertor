import { AfterViewInit, Component, DestroyRef, effect, EffectCleanupRegisterFn, EffectRef, ElementRef, inject, input } from "@angular/core";
import { BehaviorSubject, map, mergeMap, Observable, range, Subscription } from "rxjs";
import { EChartHandle, initEChart } from "../../../../common/echarts/echarts.factory";
import { ECOption } from "../../../../common/echarts/echarts.registry";
import { LatencyService } from "../../../../service/latency/latency-service";
import { ResponseStatus } from "../../../../service/latency/latency-types";

@Component({
    selector: "app-bench-bar-chart",
    imports: [],
    templateUrl: "./bench-bar-chart.html",
    styleUrl: "./bench-bar-chart.scss",
})
export class BenchBarChart implements AfterViewInit {
    // 注入
    protected readonly host = inject<ElementRef<HTMLElement>>(ElementRef<HTMLElement>);
    protected readonly destroyRef = inject(DestroyRef);
    protected readonly latencyService = inject(LatencyService);

    // 参数
    public startAction = input.required<Observable<BenchAction>>();
    public abortAction = input.required<Observable<{}>>();

    // 内部状态
    public readonly next: BehaviorSubject<BarChartSample> = new BehaviorSubject({ index: 0, state: BarChartState.PENDING, value: 0 } as BarChartSample);
    protected data: BarChartSample[] = [];
    protected windowSize: number = 32;
    protected pendingSource: number = 0;
    protected chartHandle!: EChartHandle;

    // 订阅
    protected nextSubscription: Subscription = this.next.subscribe(value => {
        if (this.data.length > value.index) {
            this.data[value.index] = value;
            this.pendingSource += 1;
        } else {
            this.data.push(value);
            this.pendingSource += 1;
        }
        if (this.pendingSource >= this.windowSize) {
            this.chartHandle.chart.setOption({
                dataset: {
                    source: this.data,
                },
            });
            this.pendingSource = 0;
        }
    });
    protected startSubscription?: Subscription;

    protected startActionOff?: EffectRef = effect((onCleanup: EffectCleanupRegisterFn) => {
        const startSubscription = this.startAction().subscribe((action) => {
            this.start(action);
        });
        onCleanup(() => {
            startSubscription.unsubscribe();
        });
    });

    protected abortActionOff?: EffectRef = effect((onCleanup: EffectCleanupRegisterFn) => {
        const abortSubscription = this.abortAction().subscribe(() => {
            this.abort();
        });
        onCleanup(() => {
            abortSubscription.unsubscribe();
        });
    });

    public ngAfterViewInit() {
        this.chartHandle = initEChart(this.host.nativeElement);
        this.chartHandle.chart.setOption(this.barOption([]));

        this.destroyRef.onDestroy(() => {
            this.chartHandle.dispose();
            this.nextSubscription?.unsubscribe();
            this.startSubscription?.unsubscribe();
            this.startActionOff?.destroy();
            this.abortActionOff?.destroy();
        });
    }

    public start(action: BenchAction) {
        this.abort();

        this.data.length = 0;
        this.chartHandle.chart.setOption({
            dataset: {
                source: this.data,
            },
        });
        this.startSubscription = range(0, action.count)
            .pipe(
                mergeMap((index) =>
                        this.latencyService.fetchWithLatency$(action.url)
                            .pipe(
                                map((latency) => {
                                    return {
                                        index,
                                        state: barChartStateFromStatus(latency.status),
                                        value: latency.totalMs,
                                    };
                                }),
                            ),
                    this.windowSize,
                ),
            )
            .subscribe((latency: BarChartSample) => {
                this.next.next(latency);
            });
    }

    public abort() {
        this.startSubscription?.unsubscribe();
        this.startSubscription = undefined;
    }

    // 柱状图
    private barOption(values: BarChartSample[]): ECOption {
        return {
            grid: { left: 0, right: 0, top: 2, bottom: 2 },
            dataset: {
                dimensions: [ "index", "state", "value" ],
                source: values,
            },
            tooltip: { trigger: "axis" },
            xAxis: { type: "category" },
            yAxis: {
                type: "value",
                // max: extent => extent.max + 1,
                max: 50,
            },
            series: [
                {
                    id: "latency",
                    type: "bar",
                    encode: { x: "index", y: "value" },
                    itemStyle: {
                        color: (p) => {
                            switch ((p.value as BarChartSample).state) {
                                case BarChartState.PENDING:
                                    return "#999999";
                                case BarChartState.OK:
                                    return "#4caf50";
                                case BarChartState.ERROR:
                                    return "#f44336";
                                case BarChartState.TIMEOUT:
                                    return "#ff9800";
                            }
                        },
                    },
                },
            ],
        };
    }
}


export enum BarChartState {
    PENDING = "pending",
    OK = "ok",
    ERROR = "error",
    TIMEOUT = "timeout",
}

function barChartStateFromStatus(status: ResponseStatus) {
    switch (status) {
        case ResponseStatus.OK:
            return BarChartState.OK;
        case ResponseStatus.ERROR:
            return BarChartState.ERROR;
        case ResponseStatus.TIMEOUT:
            return BarChartState.TIMEOUT;
        default:
            return BarChartState.PENDING;
    }
}

export interface BarChartSample {
    index: number,

    state: BarChartState,

    value: number;
}

export interface BenchAction {
    url: URL | string,

    count: number,
}
