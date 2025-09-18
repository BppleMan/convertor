export default class ApiStatus {

    constructor(
        public readonly code: number,
        public readonly message: string,
    ) {
    }

    public static deserialize(json: ApiStatus): ApiStatus {
        return new ApiStatus(json.code, json.message);
    }

    public isOk(): boolean {
        return this.code === 0;
    }

    public isError(): boolean {
        return this.code === -1;
    }

}
