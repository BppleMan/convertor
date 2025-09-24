import Cloneable from "../base/cloneable";
import Equatable from "../base/equals";
import Serializable from "../base/serializable";
import { Policy } from "./policy";

export class ConvertorUrl implements Cloneable<ConvertorUrl>, Equatable<ConvertorUrl>, Serializable {
    public url?: URL;

    public constructor(
        public type: UrlType,
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
            UrlType.deserialize(url.type),
            url.server,
            url.desc,
            url.path,
            url.query,
        );
    }
}

export class UrlType implements Cloneable<UrlType>, Equatable<UrlType>, Serializable {
    public constructor(
        public name: string,
        public policy?: Policy,
    ) {
    }

    public clone(): UrlType {
        return new UrlType(this.name, this.policy?.clone());
    }

    public equals(other?: UrlType): boolean {
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
            return new UrlType(type);
        } else {
            const name = Object.keys(type)[0];
            const policy = !!type[name] ? Policy.deserialize(type[name]) : undefined;
            return new UrlType(name, policy);
        }
    }
}
