import { JsonPipe, KeyValuePipe } from "@angular/common";
import { HttpErrorResponse } from "@angular/common/http";
import { ChangeDetectionStrategy, Component, computed, input, Signal } from "@angular/core";
import { MatCardModule } from "@angular/material/card";
import { MatChipsModule } from "@angular/material/chips";
import { MatDividerModule } from "@angular/material/divider";
import { MatExpansionModule } from "@angular/material/expansion";
import { ApiResponse } from "../../../common/response/response";
import { Title } from "../title/title";

@Component({
    selector: "app-error-view",
    imports: [ JsonPipe, KeyValuePipe, MatCardModule, MatExpansionModule, MatDividerModule, MatChipsModule, Title ],
    templateUrl: "./error-view.html",
    styleUrl: "./error-view.scss",
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ErrorView {
    error = input.required<HttpErrorResponse>();

    apiResponse: Signal<ApiResponse> = computed(() => {
        return ApiResponse.deserialize(this.error().error);
    });

    // Parse response body
    responseBody = computed(() => {
        const error = this.error().error;
        if (typeof error === "string") {
            try {
                return JSON.parse(error);
            } catch {
                return { message: error };
            }
        }
        return error || {};
    });

    // Get response message
    responseMessage = computed(() => {
        const body = this.responseBody();
        return body.message || body.error || this.error().message || "Unknown error";
    });

    // Get response headers as key-value pairs
    responseHeaders = computed(() => {
        const headers: { [key: string]: string } = {};
        this.error().headers.keys().forEach(key => {
            headers[key] = this.error().headers.get(key) || "";
        });
        return headers;
    });

    // Check if there are headers to display
    hasHeaders = computed(() => Object.keys(this.responseHeaders()).length > 0);
}
