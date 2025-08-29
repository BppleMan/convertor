import { Component, effect } from "@angular/core";
import { map, switchMap, tap } from "rxjs";
import { SampleData } from "../../../../common/echarts/echarts.options";
import { DashboardLineChart } from "../dashboard-line-chart";

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
                tap((latency) => console.log(latency.totalLatencyMs)),
                map(latency => ({ time: latency.startedAtEpochMs, value: latency.totalLatencyMs })),
            ).subscribe((latency: SampleData) => {
                this.next.next(latency);
            });
        });
    }

}
