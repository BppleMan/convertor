import { AsyncPipe } from "@angular/common";
import { ChangeDetectionStrategy, Component, inject } from "@angular/core";
import { DashboardService } from "../../service/dashboard.service";
import { ErrorView } from "../shared/error-view/error-view";
import { NoContent } from "../shared/no-content/no-content";
import { DashboardInfo } from "./dashboard-info/dashboard-info";
import { DashboardParam } from "./dashboard-param/dashboard-param";
import { DashboardSubs } from "./dashboard-subs/dashboard-subs";

@Component({
    selector: "app-dashboard",
    imports: [
        DashboardInfo,
        DashboardSubs,
        DashboardParam,
        AsyncPipe,
        NoContent,
        ErrorView,
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
