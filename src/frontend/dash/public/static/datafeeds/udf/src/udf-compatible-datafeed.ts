import { UDFCompatibleDatafeedBase } from "./udf-compatible-datafeed-base";
import { QuotesProvider } from "./quotes-provider";
import { Requester } from "./requester";
import { LimitedResponseConfiguration } from "./history-provider";

export class UDFCompatibleDatafeed extends UDFCompatibleDatafeedBase {
    public constructor(
        datafeedURL: string,
        updateFrequency: number = 10 * 1000,
        limitedServerResponse?: LimitedResponseConfiguration
    ) {
        console.log("new datafeed udf");
        let requester = new Requester();
        if ((updateFrequency < 0 || (globalThis as any).use_local) ?? false) {
            requester = new Requester({ Realtime: "false" });
        }
        const quotesProvider = new QuotesProvider(datafeedURL, requester);
        super(datafeedURL, quotesProvider, requester, updateFrequency, limitedServerResponse);
    }
}
