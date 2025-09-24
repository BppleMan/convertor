import { JsonPipe, KeyValuePipe } from "@angular/common";
import { HttpErrorResponse } from "@angular/common/http";
import { ChangeDetectionStrategy, Component, computed, input } from "@angular/core";
import { MatCardModule } from "@angular/material/card";
import { MatChipsModule } from "@angular/material/chips";
import { MatDividerModule } from "@angular/material/divider";
import { MatExpansionModule } from "@angular/material/expansion";

@Component({
    selector: "app-error-view",
    imports: [ JsonPipe, KeyValuePipe, MatCardModule, MatExpansionModule, MatDividerModule, MatChipsModule ],
    templateUrl: "./error-view.html",
    styleUrl: "./error-view.scss",
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ErrorView {
    error = input.required<HttpErrorResponse>();

    // Computed properties for better template readability
    requestMethod = computed(() => {
        // HttpErrorResponse doesn't directly contain method, but we can try to extract it
        // from the error context or default to 'Unknown'
        return "Unknown"; // Method is not available in HttpErrorResponse by default
    });
    requestUrl = computed(() => this.error().url || "Unknown");
    statusCode = computed(() => this.error().status);
    statusText = computed(() => this.error().statusText);

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
