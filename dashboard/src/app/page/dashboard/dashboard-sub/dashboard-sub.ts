import { Component } from "@angular/core";
import { MatButton } from "@angular/material/button";
import { MatFormField, MatLabel } from "@angular/material/form-field";
import { MatInput } from "@angular/material/input";
import { MatOption, MatSelect } from "@angular/material/select";
import { IconButton } from "../../shared/icon-button/icon-button";

@Component({
    selector: "app-dashboard-sub",
    imports: [
        MatFormField,
        MatLabel,
        MatInput,
        MatSelect,
        MatOption,
        MatButton,
        IconButton,
    ],
    templateUrl: "./dashboard-sub.html",
    styleUrl: "./dashboard-sub.scss",
})
export class DashboardSub {

}
