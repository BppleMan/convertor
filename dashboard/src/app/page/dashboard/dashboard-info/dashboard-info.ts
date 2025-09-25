import { CdkCopyToClipboard } from "@angular/cdk/clipboard";
import { AsyncPipe } from "@angular/common";
import { Component, inject } from "@angular/core";
import { MatChip } from "@angular/material/chips";
import { MatDivider } from "@angular/material/divider";
import { EnvService } from "../../../service/env.service";
import { IconButton } from "../../shared/icon-button/icon-button";

@Component({
    selector: "app-dashboard-info",
    imports: [
        IconButton,
        AsyncPipe,
        CdkCopyToClipboard,
        MatDivider,
        MatChip,
    ],
    templateUrl: "./dashboard-info.html",
    styleUrl: "./dashboard-info.scss",
})
export class DashboardInfo {
    private envService = inject(EnvService);
    host$ = this.envService.host.asObservable();
    userAgent$ = this.envService.userAgent.asObservable();
}

export class DashboardInfoItem {
    constructor(
        public name: string,
        public value: string,
    ) {
    }
}
