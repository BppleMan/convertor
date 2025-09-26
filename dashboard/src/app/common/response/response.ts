import { RequestSnapshot } from "./request";

export class ApiResponse<T = void> {
    constructor(
        public status: string,
        public messages: string[],
        public request: RequestSnapshot | null,
        public data?: T,
    ) {
    }

    public static deserialize<T>(json: ApiResponse<T> | any, ctor?: {
        new(...args: any[]): T;
        deserialize(json: T): T;
    }): ApiResponse<T> {
        return new ApiResponse<T>(
            json.status,
            json.messages,
            RequestSnapshot.deserialize(json.request),
            ctor?.deserialize(json.data) ?? json.data,
        );
    }

    public isOk(): boolean {
        return this.status === "ok";
    }
}
