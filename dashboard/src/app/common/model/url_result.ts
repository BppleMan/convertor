import { ConvertorUrl } from "./convertor_url";

export class UrlResult {
    public constructor(
        public raw_url: ConvertorUrl,
        public raw_profile_url: ConvertorUrl,
        public profile_url: ConvertorUrl,
        public sub_logs_url: ConvertorUrl,
        public rule_providers_url: ConvertorUrl[],
    ) {
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
}
