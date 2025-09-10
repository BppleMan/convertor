import { HttpClient } from "@angular/common/http";
import { Injectable } from "@angular/core";
import { map, Observable, tap } from "rxjs";
import ConvertorQuery from "../common/model/convertor_query";
import { UrlResult } from "../common/model/url_result";
import { ApiResponse } from "../common/response/response";
import { LatencyService } from "./latency/latency-service";
import { LatencyResult } from "./latency/latency-types";

@Injectable({
    providedIn: "root",
})
export class DashboardService {
    public static readonly HEALTH_ENDPOINT = `/actuator/healthy`;
    public static readonly REDIS_ENDPOINT = `/actuator/redis`;

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
        return this.http.get(query.subscriptionPath()).pipe(
            tap(console.log),
            map(response => ApiResponse.deserialize(response, UrlResult)),
        );
    }
}
