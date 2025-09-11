import { AsyncPipe } from "@angular/common";
import { Component, DestroyRef, inject } from "@angular/core";
import { takeUntilDestroyed } from "@angular/core/rxjs-interop";
import { FormControl, FormGroup, ReactiveFormsModule, Validators } from "@angular/forms";
import { MatButton } from "@angular/material/button";
import { MatFormField, MatLabel } from "@angular/material/form-field";
import { MatInput } from "@angular/material/input";
import { MatOption, MatSelect } from "@angular/material/select";
import { MatSlideToggle } from "@angular/material/slide-toggle";
import { StorageMap } from "@ngx-pwa/local-storage";
import {
    BehaviorSubject,
    debounceTime,
    distinctUntilChanged,
    filter,
    map,
    merge,
    Observable,
    Subject,
    switchMap,
    tap,
} from "rxjs";
import { ConvertorUrl } from "../../../common/model/convertor_url";
import { ProxyClient, SubProvider } from "../../../common/model/enums";
import { UrlResult } from "../../../common/model/url_result";
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
        AsyncPipe,
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

    urlResult = new BehaviorSubject<UrlResult>(UrlResult.empty());
    urls$: Observable<ConvertorUrl[]> = this.urlResult.pipe(
        filter((v?: UrlResult): v is UrlResult => !!v),
        map((result: UrlResult) => [
            result.raw_url,
            result.raw_profile_url,
            result.profile_url,
            result.sub_logs_url,
            ...result.rule_providers_url,
        ]),
    );
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
        this.nextSubmit.pipe(
            map(() => this.subscriptionForm.getRawValue()),
        ),
        this.subscriptionForm.valueChanges.pipe(
            debounceTime(300),
            map(() => this.subscriptionForm.getRawValue()),
            distinctUntilChanged((a, b) => JSON.stringify(a) === JSON.stringify(b)),
        ),
    )
    .pipe(
        filter(() => this.subscriptionForm.valid),
        map((payload) => <UrlParams>{
            secret: payload.secret!,
            url: payload.url!,
            client: payload.client,
            provider: payload.provider,
            interval: payload.interval,
            strict: payload.strict,
        }),
        tap((urlParams) => {
            this.storage.set("url", urlParams.url).subscribe();
            this.storage.set("secret", urlParams.secret).subscribe();
        }),
        switchMap((urlParams) => {
            const query = this.urlService.buildSubscriptionQuery(urlParams);
            return this.dashboardService.getSubscription(query);
        }),
        takeUntilDestroyed(this.destroyRef),
    )
    .subscribe(value => {
        console.log(value);
        if (value.status === 0) {
            this.urlResult.next(value.data!);
        }
    });

    submit() {
        this.nextSubmit.next();
    }

}
