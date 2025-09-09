export default class ConvertorQuery {

    public static API_SUBSCRIPTION = "api/subscription";

    public constructor(
        public client: string,
        public provider: string,
        public interval: number,
        public strict: boolean,
        public sub_url: string,
    ) {
    }

    public subscriptionPath(): string {
        return `${ConvertorQuery.API_SUBSCRIPTION}/${this}`;
    }

    public toString(): string {
        const params = new URLSearchParams();
        params.set("interval", this.interval.toString());
        params.set("strict", this.strict ? "true" : "false");
        params.set("sub_url", this.sub_url);
        return `${this.client}/${this.provider}?${params.toString()}`;
    }
}
