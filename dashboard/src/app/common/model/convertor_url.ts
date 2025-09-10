import { Policy } from "./policy";

export class ConvertorUrl {
    public url?: URL;

    public constructor(
        public type: ConvertorUrlType,
        public server: string,
        public desc: string,
        public path?: string,
        public query?: string,
    ) {
        if (server.length > 0) {
            this.url = new URL(server);
        }
        const url = this.url;
        if (url) {
            if (!!path && path.length > 0 && !!query && query.length > 0) {
                url.pathname = path;
                url.search = query;
            }
        }
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

    public static get RawUrl(): ConvertorUrl {
        return new ConvertorUrl(
            new ConvertorUrlType("Raw"),
            "",
            "Raw URL",
            "",
            "",
        );
    }

    public static get RawProfileUrl(): ConvertorUrl {
        return new ConvertorUrl(
            new ConvertorUrlType("RawProfile"),
            "",
            "Raw Profile URL",
            "",
            "",
        );
    }

    public static get ProfileUrl(): ConvertorUrl {
        return new ConvertorUrl(
            new ConvertorUrlType("Profile"),
            "",
            "Profile URL",
            "",
            "",
        );
    }

    public static get SubLogsUrl(): ConvertorUrl {
        return new ConvertorUrl(
            new ConvertorUrlType("Logs"),
            "",
            "Subscription Logs URL",
            "",
            "",
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
