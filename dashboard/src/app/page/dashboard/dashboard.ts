import { ChangeDetectionStrategy, Component } from "@angular/core";
import { BehaviorSubject, filter, interval } from "rxjs";
import { DashboardService } from "../../service/dashboard.service";
import { BenchAction } from "../shared/charts/bench-bar-chart/bench-bar-chart";
import { DashboardInfo } from "./dashboard-info/dashboard-info";
import { DashboardSub } from "./dashboard-sub/dashboard-sub";

@Component({
    selector: "app-dashboard",
    imports: [
        DashboardInfo,
        DashboardSub,
    ],
    templateUrl: "./dashboard.html",
    styleUrl: "./dashboard.scss",
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class Dashboard {
    public readonly refresh = interval(1000);

    public startBenchAction = new BehaviorSubject<BenchAction | null>(null);
    public startBenchAction$ = this.startBenchAction.pipe(
        filter(action => !!action),
    );

    public abortBenchAction = new BehaviorSubject<{} | null>(null);
    public abortBenchAction$ = this.abortBenchAction.pipe(
        filter(action => action !== null),
    );

    public startBench() {
        this.startBenchAction.next({ url: DashboardService.HEALTH_ENDPOINT, count: 100 });
    }

    public abortBench() {
        this.abortBenchAction.next({});
    }
}
