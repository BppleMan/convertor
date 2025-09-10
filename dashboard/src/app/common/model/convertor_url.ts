import Cloneable from "../base/cloneable";
import Equatable from "../base/equals";
import Serializable from "../base/serializable";
import { Policy } from "./policy";

export class ConvertorUrl implements Cloneable<ConvertorUrl>, Equatable<ConvertorUrl>, Serializable {
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

    public clone(): ConvertorUrl {
        return new ConvertorUrl(
            this.type.clone(),
            this.server,
            this.desc,
            this.path,
            this.query,
        );
    }

    public equals(other?: ConvertorUrl): boolean {
        if (!other) return false;
        return this.type.equals(other.type)
            && this.server === other.server
            && this.desc === other.desc
            && this.path === other.path;
    }

    public serialize(): any {
        return {
            type: this.type.serialize(),
            server: this.server,
            desc: this.desc,
            path: this.path,
            query: this.query,
        };
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

export class ConvertorUrlType implements Cloneable<ConvertorUrlType>, Equatable<ConvertorUrlType>, Serializable {
    public constructor(
        public name: string,
        public policy?: Policy,
    ) {
    }

    public clone(): ConvertorUrlType {
        return new ConvertorUrlType(this.name, this.policy?.clone());
    }

    public equals(other?: ConvertorUrlType): boolean {
        if (!other) return false;
        return this.name === other?.name
            && (this.policy?.equals(other?.policy) ?? false);
    }

    public serialize(): any {
        return {
            name: this.name,
            policy: this.policy,
        };
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
