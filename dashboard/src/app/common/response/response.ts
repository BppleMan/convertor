import ApiStatus from "./status";

export class ApiResponse<T = void> {
    constructor(
        public status: ApiStatus,
        public data?: T,
    ) {
    }

    public static deserialize<T>(json: ApiResponse<T> | any, ctor?: {
        new(...args: any[]): T;
        deserialize(json: T): T;
    }): ApiResponse<T> {
        return new ApiResponse<T>(
            ApiStatus.deserialize(json.status),
            ctor?.deserialize(json.data) ?? json.data,
        );
    }
}
