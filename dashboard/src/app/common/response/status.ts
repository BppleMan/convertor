export default class ApiStatus {

    constructor(
        public readonly code: number,
        public readonly messages: string[],
    ) {
    }

    public static deserialize(json: ApiStatus): ApiStatus {
        return new ApiStatus(json.code, json.messages);
    }

    public isOk(): boolean {
        return this.code === 0;
    }

    public isError(): boolean {
        return this.code === -1;
    }

}
