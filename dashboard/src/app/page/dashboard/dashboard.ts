import { ChangeDetectionStrategy, Component, inject } from "@angular/core";
import { toSignal } from "@angular/core/rxjs-interop";
import { MatButton, MatIconButton } from "@angular/material/button";
import { MatCardContent, MatCardFooter } from "@angular/material/card";
import { MatIcon } from "@angular/material/icon";
import { MatToolbar } from "@angular/material/toolbar";
import { catchError, EMPTY, filter, map, startWith, Subject, switchMap } from "rxjs";
import { DashboardService } from "../../service/dashboard-service";
import { DashboardCard } from "../shared/dashboard-card/dashboard-card";

@Component({
    selector: "app-dashboard",
    imports: [
        MatToolbar,
        MatIconButton,
        MatIcon,
        DashboardCard,
        MatCardContent,
        MatCardFooter,
        MatButton,
    ],
    templateUrl: "./dashboard.html",
    styleUrl: "./dashboard.scss",
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class Dashboard {
    readonly dashboardService: DashboardService = inject<DashboardService>(DashboardService);

    readonly refresh = new Subject<string>();
    readonly healthy = toSignal<number>(
        this.refresh.pipe(
            startWith(null),
            filter(endpoint => endpoint != null && endpoint === DashboardService.HEALTHY_ENDPOINT),
            switchMap(() => this.dashboardService.healthCheck().pipe(
                catchError(error => {
                    console.log(error);
                    return EMPTY;
                }),
                // finalize(/* 可以清理某些遗留状态 */)
            )),
            map(apiResponse => apiResponse.status),
        ),
    );

    healthCheck(event: PointerEvent) {
        this.refresh.next(DashboardService.HEALTHY_ENDPOINT);
    }
}
