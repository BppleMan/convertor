export class RequestSnapshot {
    constructor(
        public method: string,
        public scheme: string,
        public host: string,
        public uri: string,
        public headers: Map<string, string>,
    ) {
    }

    public static deserialize(json: RequestSnapshot | any): RequestSnapshot | null {
        let isRequestSnapshot = Object.hasOwn(json, "method")
            && Object.hasOwn(json, "scheme")
            && Object.hasOwn(json, "host")
            && Object.hasOwn(json, "uri")
            && Object.hasOwn(json, "headers");
        if (!isRequestSnapshot) {
            return null;
        }
        return new RequestSnapshot(
            json.method,
            json.scheme,
            json.host,
            json.uri,
            new Map(Object.entries(json.headers)),
        );
    }

    public url(): string {
        return `${this.scheme}://${this.host}${this.uri}`;
    }
}
