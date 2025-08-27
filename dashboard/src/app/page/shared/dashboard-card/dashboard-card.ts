import { ChangeDetectionStrategy, Component, CUSTOM_ELEMENTS_SCHEMA, input } from "@angular/core";
import { MatCard, MatCardHeader } from "@angular/material/card";

@Component({
    selector: "app-dashboard-card",
    imports: [
        MatCard,
        MatCardHeader,
    ],
    templateUrl: "./dashboard-card.html",
    styleUrl: "./dashboard-card.scss",
    changeDetection: ChangeDetectionStrategy.OnPush,
    schemas: [ CUSTOM_ELEMENTS_SCHEMA ],
})
export class DashboardCard {
    name = input.required<string>();
}
