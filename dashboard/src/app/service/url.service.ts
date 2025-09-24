import { Injectable } from "@angular/core";
import ConvertorQuery from "../common/model/convertor_query";
import { Crypto_xchachaService } from "./crypto_xchacha.service";


@Injectable({
    providedIn: "root",
})
export class UrlService {
    constructor(
        public crypto: Crypto_xchachaService,
    ) {
    }

    public buildSubscriptionQuery(params: UrlParams): ConvertorQuery {
        const { secret, url, client, interval, strict } = params;
        const sub_url = this.crypto.encrypt(secret, url);
        return new ConvertorQuery(
            client,
            interval,
            strict,
            sub_url,
        );
    }
}

export interface UrlParams {
    secret: string;

    url: string;

    client: string;

    interval: number;

    strict: boolean;
}
