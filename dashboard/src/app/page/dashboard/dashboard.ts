import { ChangeDetectionStrategy, Component } from "@angular/core";
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
}
