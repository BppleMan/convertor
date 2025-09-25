import { HttpErrorResponse } from "@angular/common/http";
import { ChangeDetectionStrategy, Component } from "@angular/core";
import { BehaviorSubject } from "rxjs";
import { DashboardInfo } from "./dashboard-info/dashboard-info";
import { DashboardParam } from "./dashboard-param/dashboard-param";
import { DashboardSubs } from "./dashboard-subs/dashboard-subs";

@Component({
    selector: "app-dashboard",
    imports: [
        DashboardInfo,
        DashboardSubs,
        DashboardParam,
    ],
    templateUrl: "./dashboard.html",
    styleUrl: "./dashboard.scss",
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class Dashboard {
    loading: BehaviorSubject<boolean> = new BehaviorSubject<boolean>(false);
    error: BehaviorSubject<HttpErrorResponse | undefined> = new BehaviorSubject<HttpErrorResponse | undefined>(undefined);

}
