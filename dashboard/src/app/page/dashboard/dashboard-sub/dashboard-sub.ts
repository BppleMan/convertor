import { CdkCopyToClipboard } from "@angular/cdk/clipboard";
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
    catchError,
    debounceTime,
    defer,
    distinctUntilChanged,
    EMPTY,
    exhaustMap,
    filter,
    finalize,
    forkJoin,
    map,
    merge,
    Observable,
    shareReplay,
    Subject,
    switchMap,
    takeUntil,
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
        CdkCopyToClipboard,
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

    loading: BehaviorSubject<boolean> = new BehaviorSubject<boolean>(false);
    submit$: Subject<void> = new Subject<void>();
    cancel$ = new Subject<void>();
    error$ = new Subject<string>();
    params$ = this.subscriptionForm.valueChanges.pipe(
        debounceTime(300),
        map(() => this.subscriptionForm.getRawValue()),
        distinctUntilChanged(this.deepEqual),
        filter(() => this.subscriptionForm.valid),
        map(payload => this.toUrlParams(payload)),
        shareReplay({ bufferSize: 1, refCount: true }),
    );

    formRestoreSub = merge(
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

    storageSub = this.params$.pipe(
        switchMap(p =>
            forkJoin([
                this.storage.set("url", p.url),
                this.storage.set("secret", p.secret),
            ]).pipe(
                catchError(() => EMPTY),
            ),
        ),
        takeUntilDestroyed(this.destroyRef),
    ).subscribe();

    requestSub = merge(
        this.params$,
        // 手动：点击提交时直接抓取当前 rawValue（不依赖 params$ 是否已发过值）
        this.submit$.pipe(
            map(() => this.subscriptionForm.getRawValue()),
            filter(() => this.subscriptionForm.valid),
            map(payload => this.toUrlParams(payload)),
        ),
    )
    .pipe(
        exhaustMap((urlParams) => {
            return defer(() => {
                // 请求开始：锁表单 & 开 loading
                this.subscriptionForm.disable({ emitEvent: false });
                this.loading.next(true);

                const query = this.urlService.buildSubscriptionQuery(urlParams);
                return this.dashboardService.getSubscription(query).pipe(
                    // 主动取消当前请求
                    takeUntil(this.cancel$),
                    // 错误只在 HTTP 内部处理，吞掉，不打断主流
                    catchError(err => {
                        // this.error$.next(extractHttpError(err));
                        console.error(err);
                        return EMPTY;
                    }),

                    // 结束（成功/失败/取消）：解锁 & 关 loading
                    finalize(() => {
                        this.subscriptionForm.enable({ emitEvent: false });
                        this.loading.next(false);
                    }),
                );
            });
        }),
        takeUntilDestroyed(this.destroyRef),
    )
    .subscribe(value => {
        console.log(value);
        if (value.status === 0) {
            this.urlResult.next(value.data!);
        } else {
            // 处理非网络的业务型错误
        }
    });

    submit() {
        this.submit$.next();
    }

    cancel() {
        this.cancel$.next();
    }

    toUrlParams(payload: {
        secret: string | null,
        url: string | null,
        client: string,
        provider: string,
        interval: number,
        strict: boolean,
    }): UrlParams {
        return {
            secret: payload.secret!,
            url: payload.url!,
            client: payload.client,
            provider: payload.provider,
            interval: payload.interval,
            strict: payload.strict,
        };
    }

    deepEqual<T>(a: T, b: T): boolean {
        return JSON.stringify(a) === JSON.stringify(b);
    }
}
