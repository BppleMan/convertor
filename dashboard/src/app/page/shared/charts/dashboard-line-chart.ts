import { AfterViewInit, DestroyRef, Directive, ElementRef, inject, input } from "@angular/core";
import { BehaviorSubject, Observable, scan, Subscription } from "rxjs";
import { EChartHandle, initEChart } from "../../../common/echarts/echarts.factory";
import { lineOption, SampleData } from "../../../common/echarts/echarts.options";
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
    public readonly windowMs = input<number>(30_000);
    public readonly shiftDelayMs = input<number>(2_000);
    public readonly targetFps = input<Hz>(144);

    // 内部状态
    public readonly next: BehaviorSubject<SampleData> = new BehaviorSubject({ time: Date.now(), value: 0 });
    protected chartHandle!: EChartHandle;
    protected pendingSource: SampleData[] | null = [ { time: Date.now(), value: 0 } ];
    protected subscribeNext: Subscription | null = null;
    protected offTimeTick: (() => void) | null = null;

    ngAfterViewInit(): void {
        this.chartHandle = initEChart(this.host.nativeElement);
        this.chartHandle.chart.setOption(lineOption([]));

        this.subscribeNext = this.next.pipe(
            scan((acc: SampleData[], value) => {
                acc.push(value);
                if (acc.length > 33) {
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
        const windowMs = this.windowMs() ?? 30_000;
        const shiftDelayMs = this.shiftDelayMs() ?? 2_000;
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

}
