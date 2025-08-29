import { ChangeDetectionStrategy, Component } from "@angular/core";
import { MatIconButton } from "@angular/material/button";
import { MatCardContent, MatCardFooter } from "@angular/material/card";
import { MatIcon } from "@angular/material/icon";
import { MatToolbar } from "@angular/material/toolbar";
import { interval } from "rxjs";
import { HealthChart } from "../shared/charts/health-chart/health-chart";
import { RedisChart } from "../shared/charts/redis-chart/redis-chart";
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
        HealthChart,
        RedisChart,
    ],
    templateUrl: "./dashboard.html",
    styleUrl: "./dashboard.scss",
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class Dashboard {
    public readonly refresh = interval(1000);


    healthCheck(event: PointerEvent) {
        // this.refresh.next(DashboardService.HEALTHY_ENDPOINT);
    }
}
