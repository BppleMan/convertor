import { Component, input } from "@angular/core";
import { MatCard, MatCardHeader } from "@angular/material/card";

@Component({
    selector: "app-dashboard-panel",
    imports: [
        MatCard,
        MatCardHeader,
    ],
    templateUrl: "./dashboard-panel.html",
    styleUrl: "./dashboard-panel.scss",
})
export class DashboardPanel {
    name = input.required<string>();

}
