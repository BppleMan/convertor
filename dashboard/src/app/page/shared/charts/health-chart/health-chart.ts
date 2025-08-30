import { Component, effect } from "@angular/core";
import { map, switchMap } from "rxjs";
import { DashboardLineChart, LineChartSample } from "../dashboard-line-chart";

@Component({
    selector: "app-health-chart",
    imports: [],
    templateUrl: "./health-chart.html",
    styleUrl: "./health-chart.scss",
})
export class HealthChart extends DashboardLineChart {

    constructor() {
        super();
        effect(() => {
            this.valueTicker().pipe(
                switchMap(() => this.dashboardService.healthLatency()),
                map(latency => ({ time: latency.startedAt, value: latency.totalMs })),
            ).subscribe((latency: LineChartSample) => {
                this.next.next(latency);
            });
        });
    }

}
