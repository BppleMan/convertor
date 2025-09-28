import { AsyncPipe } from "@angular/common";
import { ChangeDetectionStrategy, Component, inject } from "@angular/core";
import { MatCardContent, MatCardHeader, MatCardTitle } from "@angular/material/card";
import { DashboardService } from "../../service/dashboard.service";
import { NoContent } from "../shared/no-content/no-content";
import { DashboardError } from "./dashboard-error/dashboard-error";
import { DashboardPanel } from "./dashboard-panel/dashboard-panel";
import { DashboardParam } from "./dashboard-param/dashboard-param";
import { DashboardSubs } from "./dashboard-subs/dashboard-subs";

@Component({
    selector: "app-dashboard",
    imports: [
        DashboardSubs,
        DashboardParam,
        AsyncPipe,
        NoContent,
        DashboardPanel,
        MatCardHeader,
        MatCardContent,
        MatCardTitle,
        DashboardError,
    ],
    templateUrl: "./dashboard.html",
    styleUrl: "./dashboard.scss",
    changeDetection: ChangeDetectionStrategy.OnPush,
    providers: [
        DashboardService,
    ],
})
export class Dashboard {
    dashboardService: DashboardService = inject(DashboardService);

    error$ = this.dashboardService.error$;
    data$ = this.dashboardService.urlResult$;
}
