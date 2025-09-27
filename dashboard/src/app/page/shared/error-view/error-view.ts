import {ChangeDetectionStrategy, Component, computed, input, Signal} from "@angular/core";
import {MatCardModule} from "@angular/material/card";
import {MatChipsModule} from "@angular/material/chips";
import {MatDividerModule} from "@angular/material/divider";
import {MatExpansionModule} from "@angular/material/expansion";
import {DashboardHttpError} from "../../../common/model/dashboard-http-error";
import {RequestSnapshot} from "../../../common/response/request";
import {ApiResponse} from "../../../common/response/response";
import {Title} from "../title/title";

@Component({
    selector: "app-error-view",
    imports: [MatCardModule, MatExpansionModule, MatDividerModule, MatChipsModule, Title],
    templateUrl: "./error-view.html",
    styleUrl: "./error-view.scss",
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ErrorView {
    dashboardHttpError = input.required<DashboardHttpError>({alias: "error"});

    httpErrorResponse = computed(() => this.dashboardHttpError().cause);

    apiResponse: Signal<ApiResponse> = computed(() => {
        return ApiResponse.deserialize(this.httpErrorResponse().error);
    });

    clientRequest: Signal<RequestSnapshot | null> = computed(() => {
        const url = this.httpErrorResponse().url;
        if (!url) {
            return null;
        }
        const parsedUrl = new URL(url);
        const headers = new Map<string, string>();
        this.httpErrorResponse().headers.keys().forEach(key => {
            headers.set(key, this.httpErrorResponse().headers.get(key) ?? "");
        });
        return new RequestSnapshot(
            this.dashboardHttpError().method,
            parsedUrl.protocol,
            parsedUrl.host,
            parsedUrl.pathname + parsedUrl.search,
            headers,
        );
    });

    serverRequest: Signal<RequestSnapshot | null> = computed(() => this.apiResponse().request);

    causeMessages = computed(() => {
        return this.apiResponse().messages.slice(1);
    });
}
