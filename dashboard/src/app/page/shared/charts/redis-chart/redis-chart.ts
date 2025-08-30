import { Component, effect } from "@angular/core";
import { map, switchMap } from "rxjs";
import { DashboardLineChart, LineChartSample } from "../dashboard-line-chart";

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
                map(latency => ({ time: latency.startedAt, value: latency.totalMs })),
            ).subscribe((latency: LineChartSample) => {
                this.next.next(latency);
            });
        });
    }

}
