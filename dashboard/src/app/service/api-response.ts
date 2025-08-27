export default class ApiResponse<T> {
    constructor(
        public status: number,
        public message?: string,
        public data?: T,
    ) {
    }

    public static deserialize<T>(json: ApiResponse<T>): ApiResponse<T> {
        return new ApiResponse<T>(json.status, json.message, json.data);
    }
}
