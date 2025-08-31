import { Component } from "@angular/core";
import { IconButton } from "../../shared/icon-button/icon-button";

@Component({
    selector: "app-dashboard-info",
    imports: [
        IconButton,
    ],
    templateUrl: "./dashboard-info.html",
    styleUrl: "./dashboard-info.scss",
})
export class DashboardInfo {

}

export class DashboardInfoItem {
    constructor(
        public name: string,
        public value: string,
    ) {
    }
}
