import { HttpClient, HttpErrorResponse } from "@angular/common/http";
import { Injectable } from "@angular/core";
import { BehaviorSubject, catchError, EMPTY, finalize, map, Observable, tap } from "rxjs";
import ConvertorQuery from "../common/model/convertor_query";
import { UrlResult } from "../common/model/url_result";
import { ApiResponse } from "../common/response/response";
import { LatencyService } from "./latency/latency-service";
import { LatencyResult } from "./latency/latency-types";

@Injectable()
export class DashboardService {
    public static readonly HEALTH_ENDPOINT = `/actuator/healthy`;
    public static readonly REDIS_ENDPOINT = `/actuator/redis`;

    loading: BehaviorSubject<boolean> = new BehaviorSubject<boolean>(false);
    loading$ = this.loading.asObservable();

    error: BehaviorSubject<HttpErrorResponse | undefined> = new BehaviorSubject<HttpErrorResponse | undefined>(undefined);
    error$ = this.error.asObservable();

    urlResult = new BehaviorSubject<UrlResult | undefined>(undefined);
    urlResult$ = this.urlResult.asObservable();

    public constructor(
        private http: HttpClient,
        private latencyService: LatencyService,
    ) {
    }

    public healthCheck(): Observable<ApiResponse> {
        return this.http.get<ApiResponse>(DashboardService.HEALTH_ENDPOINT)
            .pipe(
                map(response => ApiResponse.deserialize(response)),
            );
    }

    public async healthLatency(): Promise<LatencyResult> {
        return await this.latencyService.fetchWithLatency(DashboardService.HEALTH_ENDPOINT);
    }

    public redisCheck(): Observable<ApiResponse> {
        return this.http.get<ApiResponse>(DashboardService.REDIS_ENDPOINT)
            .pipe(
                map(response => ApiResponse.deserialize(response)),
            );
    }

    public async redisLatency(): Promise<LatencyResult> {
        return await this.latencyService.fetchWithLatency(DashboardService.REDIS_ENDPOINT);
    }

    public getSubscription(query: ConvertorQuery): Observable<ApiResponse<UrlResult>> {
        this.loading.next(true);
        return this.http.get(query.subscriptionPath()).pipe(
            tap(console.log),
            map(response => ApiResponse.deserialize(response, UrlResult)),
            // 请求成功时清除错误信息
            tap(response => {
                this.error.next(undefined);
                if (response.isOk()) {
                    this.urlResult.next(response.data!);
                } else {
                    // TODO(处理非网络的业务型错误)
                }
                return response;
            }),
            // 错误只在 HTTP 内部处理，吞掉，不打断主流
            catchError((err: HttpErrorResponse) => {
                console.log(err);
                this.error.next(err);
                return EMPTY;
            }),
            // 结束（成功/失败/取消）：关 loading
            finalize(() => {
                this.loading.next(false);
            }),
        );
    }
}
