import Cloneable from "../base/cloneable";
import Equatable from "../base/equals";
import Serializable from "../base/serializable";
import { ConvertorUrl } from "./convertor_url";

export class UrlResult implements Cloneable<UrlResult>, Equatable<UrlResult>, Serializable {
    public constructor(
        public raw_url: ConvertorUrl,
        public raw_profile_url: ConvertorUrl,
        public profile_url: ConvertorUrl,
        public sub_logs_url: ConvertorUrl,
        public rule_providers_url: ConvertorUrl[],
    ) {
    }

    public static empty(): UrlResult {
        return new UrlResult(
            ConvertorUrl.RawUrl,
            ConvertorUrl.RawProfileUrl,
            ConvertorUrl.ProfileUrl,
            ConvertorUrl.SubLogsUrl,
            [],
        );
    }

    public static deserialize(result: UrlResult) {
        return new UrlResult(
            ConvertorUrl.deserialize(result.raw_url),
            ConvertorUrl.deserialize(result.raw_profile_url),
            ConvertorUrl.deserialize(result.profile_url),
            ConvertorUrl.deserialize(result.sub_logs_url),
            result.rule_providers_url.map(ConvertorUrl.deserialize),
        );
    }

    public clone(): UrlResult {
        return new UrlResult(
            this.raw_url.clone(),
            this.raw_profile_url.clone(),
            this.profile_url.clone(),
            this.sub_logs_url.clone(),
            this.rule_providers_url.map((rp) => rp.clone()),
        );
    }

    public equals(other?: UrlResult): boolean {
        if (!other) return false;
        return this.raw_url.equals(other.raw_url)
            && this.raw_profile_url.equals(other.raw_profile_url)
            && this.profile_url.equals(other.profile_url)
            && this.sub_logs_url.equals(other.sub_logs_url)
            && this.rule_providers_url.length === other.rule_providers_url.length
            && this.rule_providers_url.every((rp, index) => rp.equals(other.rule_providers_url[index]));

    }

    public serialize(): any {
        return {
            raw_url: this.raw_url.serialize(),
            raw_profile_url: this.raw_profile_url.serialize(),
            profile_url: this.profile_url.serialize(),
            sub_logs_url: this.sub_logs_url.serialize(),
            rule_providers_url: this.rule_providers_url.map((rp) => rp.serialize()),
        };
    }

}
