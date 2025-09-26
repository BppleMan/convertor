import { Component } from "@angular/core";
import { ReactiveFormsModule } from "@angular/forms";
import { MatDivider } from "@angular/material/divider";
import { Title } from "../../shared/title/title";

@Component({
    selector: "app-dashboard-subs",
    imports: [
        ReactiveFormsModule,
        MatDivider,
        Title,
    ],
    templateUrl: "./dashboard-subs.html",
    styleUrl: "./dashboard-subs.scss",
})
export class DashboardSubs {

}
