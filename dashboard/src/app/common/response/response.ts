export class ApiResponse<T = void> {
    constructor(
        public status: number,
        public message?: string,
        public data?: T,
    ) {
    }

    public static deserialize<T>(json: ApiResponse<T> | any, ctor?: {
        new(...args: any[]): T;
        deserialize(json: T): T;
    }): ApiResponse<T> {
        return new ApiResponse<T>(json.status, json.message, ctor?.deserialize(json.data) ?? json.data);
    }
}
