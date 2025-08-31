import { ChangeDetectionStrategy, Component } from "@angular/core";
import { MatButton, MatIconButton } from "@angular/material/button";
import { MatCardContent, MatCardFooter } from "@angular/material/card";
import { MatIcon } from "@angular/material/icon";
import { MatToolbar } from "@angular/material/toolbar";
import { BehaviorSubject, filter, interval } from "rxjs";
import { DashboardService } from "../../service/dashboard-service";
import { ActuatorLineChart } from "../shared/charts/actuator-line-chart/actuator-line-chart";
import { BenchAction, BenchBarChart } from "../shared/charts/bench-bar-chart/bench-bar-chart";
import { DashboardPanel } from "./dashboard-panel/dashboard-panel";

@Component({
    selector: "app-dashboard",
    imports: [
        MatToolbar,
        MatIconButton,
        MatIcon,
        MatCardContent,
        MatCardFooter,
        DashboardPanel,
        BenchBarChart,
        MatButton,
        ActuatorLineChart,
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
