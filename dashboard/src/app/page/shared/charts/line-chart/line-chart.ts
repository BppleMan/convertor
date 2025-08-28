import { AfterViewInit, Component, DestroyRef, ElementRef, HostListener, inject, input } from "@angular/core";
import { EChartHandle, initEChart } from "../../../../common/echarts/echarts.factory";
import { lineOption } from "../../../../common/echarts/echarts.options";
import { Hz, TickerService } from "../../../../service/ticker-service";

@Component({
    selector: "app-line-chart",
    imports: [],
    templateUrl: "./line-chart.html",
    styleUrl: "./line-chart.scss",
})
export class LineChart<T extends SampleData> implements AfterViewInit {
    private host = inject<ElementRef<HTMLElement>>(ElementRef<HTMLElement>);
    private destroyRef = inject(DestroyRef);
    private tickerService = inject(TickerService);

    data = input.required<T[]>();
    windowMs = input<number>(30_000);
    shiftDelayMs = input<number>(2_000);
    targetFps = input<Hz>(144);

    private chartHandle?: EChartHandle;
    private offTick: (() => void) | null = null;

    ngAfterViewInit(): void {
        console.log("ngAfterViewInit");
        console.log(this.host);
        console.log(this.host.nativeElement.clientWidth, this.host.nativeElement.clientHeight);
        this.chartHandle = initEChart(this.host.nativeElement);

        this.destroyRef.onDestroy(() => {
            if (this.offTick) {
                this.offTick();
                this.offTick = null;
            }
        });
    }

    @HostListener("click")
    toggle(): void {
        // if (this.offTick) {
        //     this.offTick();
        //     this.offTick = null;
        // } else {
        //     this.offTick = this.tickerService.onTick(this.onTick.bind(this));
        // }
        this.chartHandle?.chart.setOption(lineOption([ [ Date.now(), 5 ] ]));
        console.log(this.host.nativeElement.clientWidth, this.host.nativeElement.clientHeight);
    }

    onTick(nowMs: number, dtMs: number) {
        // console.log("tick", nowMs, dtMs);
    }
}

export interface SampleData {
    sentAt: number;
    latency: number;
}
