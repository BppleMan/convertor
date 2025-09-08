import { Component } from "@angular/core";
import { FormControl, FormGroup, ReactiveFormsModule, Validators } from "@angular/forms";
import { MatButton } from "@angular/material/button";
import { MatFormField, MatLabel } from "@angular/material/form-field";
import { MatInput } from "@angular/material/input";
import { MatOption, MatSelect } from "@angular/material/select";
import { MatSlideToggle } from "@angular/material/slide-toggle";
import { ProxyClient, SubProvider } from "../../../common/model/enums";
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
        MatSlideToggle,
        ReactiveFormsModule,
    ],
    templateUrl: "./dashboard-sub.html",
    styleUrl: "./dashboard-sub.scss",
})
export class DashboardSub {
    protected subscriptionForm = new FormGroup({
        secret: new FormControl<string | null>(null, [ Validators.required ]),
        url: new FormControl<string | null>(null, [ Validators.required ]),
        client: new FormControl<string>(ProxyClient.Surge.toLowerCase(), { nonNullable: true }),
        provider: new FormControl<string>(SubProvider.BosLife.toLowerCase(), { nonNullable: true }),
        interval: new FormControl<number>(43200, {
            nonNullable: true,
            validators: [],
        }),
        strict: new FormControl<boolean>(true),
    });

    protected providers: SubProvider[] = Object.values(SubProvider);

    protected clients: ProxyClient[] = Object.values(ProxyClient);

}
