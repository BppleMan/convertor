export class ApiResponse<T = void> {
    constructor(
        public status: number,
        public message?: string,
        public data?: T,
    ) {
    }

    public static deserialize<T>(json: ApiResponse<T> | any): ApiResponse<T> {
        return new ApiResponse<T>(json.status, json.message, json.data);
    }
}
