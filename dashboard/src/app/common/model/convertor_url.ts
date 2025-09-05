import { Policy } from "./policy";

export class ConvertorUrl {
    public url: URL;

    public constructor(
        public type: ConvertorUrlType,
        public server: string,
        public desc: string,
        public path: string,
        public query: string,
    ) {
        this.url = new URL(`${path}?${query}`, server);
    }

    public static deserialize(url: ConvertorUrl) {
        return new ConvertorUrl(
            ConvertorUrlType.deserialize(url.type),
            url.server,
            url.desc,
            url.path,
            url.query,
        );
    }
}

export class ConvertorUrlType {
    public constructor(
        public name: string,
        public policy?: Policy,
    ) {
    }

    public static deserialize(type: any) {
        if (typeof type === "string") {
            return new ConvertorUrlType(type);
        } else {
            const name = Object.keys(type)[0];
            const policy = !!type[name] ? Policy.deserialize(type[name]) : undefined;
            return new ConvertorUrlType(name, policy);
        }
    }
}
