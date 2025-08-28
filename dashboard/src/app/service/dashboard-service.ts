import { HttpClient } from "@angular/common/http";
import { Injectable } from "@angular/core";
import { map, Observable } from "rxjs";
import { ApiResponse } from "../common/response/response";

@Injectable({
    providedIn: "root",
})
export class DashboardService {
    public static readonly HEALTHY_ENDPOINT = "http://localhost:8080/healthy";

    public constructor(
        private http: HttpClient,
    ) {
    }

    public healthCheck(): Observable<ApiResponse<never>> {
        return this.http.get<ApiResponse<never>>(DashboardService.HEALTHY_ENDPOINT)
            .pipe(
                map(response => ApiResponse.deserialize(response)),
            );
    }
}
