import { Component, effect } from "@angular/core";
import { map, switchMap } from "rxjs";
import { SampleData } from "../../../../common/echarts/echarts.options";
import { DashboardLineChart } from "../dashboard-line-chart";

@Component({
    selector: "app-redis-chart",
    imports: [],
    templateUrl: "./redis-chart.html",
    styleUrl: "./redis-chart.scss",
})
export class RedisChart extends DashboardLineChart {

    constructor() {
        super();
        effect(() => {
            this.valueTicker().pipe(
                switchMap(() => this.dashboardService.redisLatency()),
                map(latency => ({ time: latency.startedAtEpochMs, value: latency.totalLatencyMs })),
            ).subscribe((latency: SampleData) => {
                this.next.next(latency);
            });
        });
    }

}
