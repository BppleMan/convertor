import { Component, DestroyRef, inject } from "@angular/core";
import { takeUntilDestroyed } from "@angular/core/rxjs-interop";
import { FormControl, FormGroup, ReactiveFormsModule, Validators } from "@angular/forms";
import { MatButton } from "@angular/material/button";
import { MatFormField, MatLabel } from "@angular/material/form-field";
import { MatInput } from "@angular/material/input";
import { MatOption, MatSelect } from "@angular/material/select";
import { MatSlideToggle } from "@angular/material/slide-toggle";
import { StorageMap } from "@ngx-pwa/local-storage";
import { debounceTime, distinctUntilChanged, filter, map, merge, Subject, switchMap } from "rxjs";
import { ProxyClient, SubProvider } from "../../../common/model/enums";
import { DashboardService } from "../../../service/dashboard.service";
import { UrlParams, UrlService } from "../../../service/url.service";
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
    providers: SubProvider[] = Object.values(SubProvider);

    clients: ProxyClient[] = Object.values(ProxyClient);

    destroyRef: DestroyRef = inject(DestroyRef);

    urlService: UrlService = inject(UrlService);

    dashboardService = inject(DashboardService);

    storage = inject(StorageMap);

    subscriptionForm = new FormGroup({
        secret: new FormControl<string | null>(null, {
            validators: [ Validators.required ],
            updateOn: "submit",
        }),
        url: new FormControl<string | null>(null, {
            validators: [ Validators.required ],
            updateOn: "submit",
        }),
        interval: new FormControl<number>(43200, {
            nonNullable: true,
            validators: [ Validators.required ],
            updateOn: "submit",
        }),
        client: new FormControl<string>(ProxyClient.Surge.toLowerCase(), { nonNullable: true }),
        provider: new FormControl<string>(SubProvider.BosLife.toLowerCase(), { nonNullable: true }),
        strict: new FormControl<boolean>(true, { nonNullable: true }),
    });

    nextSubmit: Subject<void> = new Subject();

    storageSubscription = merge(
        this.storage.get("url").pipe(
            map(value => typeof value === "string" ? value : undefined),
            map((value?: string) => ({ url: value, secret: undefined })),
        ),
        this.storage.get("secret").pipe(
            map(value => typeof value === "string" ? value : undefined),
            map((value?: string) => ({ url: undefined, secret: value })),
        ),
    )
    .pipe(
        takeUntilDestroyed(this.destroyRef),
    )
    .subscribe((value) => {
        console.log(value);
        if (!value.url) {
            delete value.url;
        }
        if (!value.secret) {
            delete value.secret;
        }
        this.subscriptionForm.patchValue(value, { emitEvent: false });
    });

    submitSubscription = merge(
        this.nextSubmit.asObservable(),
        this.subscriptionForm.valueChanges.pipe(),
    )
    .pipe(
        debounceTime(300),
        map(() => this.subscriptionForm.getRawValue()),
        distinctUntilChanged((a, b) => JSON.stringify(a) === JSON.stringify(b)),
        filter(() => this.subscriptionForm.valid),
        switchMap((payload) => {
            const urlParams: UrlParams = {
                secret: payload.secret!,
                url: payload.url!,
                client: payload.client,
                provider: payload.provider,
                interval: payload.interval,
                strict: payload.strict,
            };
            this.storage.set("url", urlParams.url).subscribe();
            this.storage.set("secret", urlParams.secret).subscribe();
            const query = this.urlService.buildSubscriptionQuery(urlParams);
            // return of(query);
            // this.submitting = true;
            // this.error = null;
            // return fakeSubmit(payload).pipe(
            //     tap(() => this.subscriptionForm.markAsPristine()),
            //     catchError(err => {
            //         this.error = err?.message ?? "Submit failed";
            //         return of(null);
            //     }),
            //     tap(() => this.submitting = false),
            // );
            return this.dashboardService.getSubscription(query);
        }),
        takeUntilDestroyed(this.destroyRef),
    )
    .subscribe(value => {
        console.log(value);
    });

    submit() {
        this.nextSubmit.next();
    }

}
