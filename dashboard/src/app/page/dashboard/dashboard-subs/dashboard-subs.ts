import { CdkCopyToClipboard } from "@angular/cdk/clipboard";
import { AsyncPipe } from "@angular/common";
import { Component } from "@angular/core";
import { ReactiveFormsModule } from "@angular/forms";
import { MatDivider } from "@angular/material/divider";
import { ErrorView } from "../../shared/error-view/error-view";
import { IconButton } from "../../shared/icon-button/icon-button";

@Component({
    selector: "app-dashboard-subs",
    imports: [
        IconButton,
        ReactiveFormsModule,
        AsyncPipe,
        CdkCopyToClipboard,
        ErrorView,
        MatDivider,
    ],
    templateUrl: "./dashboard-subs.html",
    styleUrl: "./dashboard-subs.scss",
})
export class DashboardSubs {

}
