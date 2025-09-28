import { Component } from "@angular/core";
import { MAT_CARD_CONFIG, MatCard } from "@angular/material/card";

@Component({
    selector: "app-dashboard-panel",
    imports: [
        MatCard,
    ],
    templateUrl: "./dashboard-panel.html",
    styleUrl: "./dashboard-panel.scss",
    providers: [
        {
            provide: MAT_CARD_CONFIG,
            useValue: { appearance: "raised" },
        },
    ],
})
export class DashboardPanel {

}
