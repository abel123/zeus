import {
    DatafeedErrorCallback,
    DOMCallback,
    GetMarksCallback,
    HistoryCallback,
    IDatafeedChartApi,
    IDatafeedQuotesApi,
    IExternalDatafeed,
    LibrarySymbolInfo,
    Mark,
    OnReadyCallback,
    PeriodParams,
    QuotesCallback,
    QuotesErrorCallback,
    ResolutionString,
    ResolveCallback,
    SearchSymbolsCallback,
    ServerTimeCallback,
    SubscribeBarsCallback,
    SymbolResolveExtension,
    TimescaleMark,
} from "@/public/static/charting_library/charting_library";
import { UDFCompatibleDatafeed } from "@/public/static/datafeeds/udf/src/udf-compatible-datafeed";

export enum State {
    Origin,
    Replay,
}

export class DataFeedWrapper implements IDatafeedChartApi, IExternalDatafeed, IDatafeedQuotesApi {
    datafeed_original: UDFCompatibleDatafeed;
    datafeed_replayer: UDFCompatibleDatafeed;
    state: State;
    listenerSet: Set<string> = new Set<string>();
    speed: string = "1000";
    replay_time: number | null = null;
    constructor(datafeedURL: string, updateFrequency?: number) {
        this.datafeed_original = new UDFCompatibleDatafeed(datafeedURL, updateFrequency);
        this.datafeed_replayer = new UDFCompatibleDatafeed(datafeedURL, -1);
        this.state = State.Origin;
    }

    bar_replay_set_speed(speed: string) {
        this.speed = speed;
    }
    set_replay_state(time: number) {
        this.replay_time = time;
        this.state = State.Replay;
        this.listenerSet.forEach((val) => {
            this.datafeed_original.unsubscribeBars(val);
        });
    }
    bar_replay_start() {}
    bar_replay_stop() {}
    bar_replay_step() {}
    getQuotes(symbols: string[], onDataCallback: QuotesCallback, onErrorCallback: QuotesErrorCallback): void {
        if (this.state == State.Origin) {
            return this.datafeed_original.getQuotes(symbols, onDataCallback, onErrorCallback);
        } else {
            return this.datafeed_replayer.getQuotes(symbols, onDataCallback, onErrorCallback);
        }
    }
    subscribeQuotes(
        symbols: string[],
        fastSymbols: string[],
        onRealtimeCallback: QuotesCallback,
        listenerGUID: string
    ): void {
        if (this.state == State.Origin) {
            return this.datafeed_original.subscribeQuotes(symbols, fastSymbols, onRealtimeCallback, listenerGUID);
        } else {
            return this.datafeed_replayer.subscribeQuotes(symbols, fastSymbols, onRealtimeCallback, listenerGUID);
        }
    }
    unsubscribeQuotes(listenerGUID: string): void {
        if (this.state == State.Origin) {
            return this.datafeed_original.unsubscribeQuotes(listenerGUID);
        } else {
            return this.datafeed_replayer.unsubscribeQuotes(listenerGUID);
        }
    }
    onReady(callback: OnReadyCallback): void {
        this.datafeed_original.onReady(callback);
        this.datafeed_replayer.onReady(callback);
    }
    getMarks?(
        symbolInfo: LibrarySymbolInfo,
        from: number,
        to: number,
        onDataCallback: GetMarksCallback<Mark>,
        resolution: ResolutionString
    ): void {
        if (this.state == State.Origin) {
            return this.datafeed_original.getMarks(symbolInfo, from, to, onDataCallback, resolution);
        } else {
            return this.datafeed_replayer.getMarks(symbolInfo, from, to, onDataCallback, resolution);
        }
    }
    getTimescaleMarks?(
        symbolInfo: LibrarySymbolInfo,
        from: number,
        to: number,
        onDataCallback: GetMarksCallback<TimescaleMark>,
        resolution: ResolutionString
    ): void {
        if (this.state == State.Origin) {
            return this.datafeed_original.getTimescaleMarks(symbolInfo, from, to, onDataCallback, resolution);
        } else {
            return this.datafeed_replayer.getTimescaleMarks(symbolInfo, from, to, onDataCallback, resolution);
        }
    }
    getServerTime?(callback: ServerTimeCallback): void {
        if (this.state == State.Origin) {
            return this.datafeed_original.getServerTime(callback);
        } else {
            return this.datafeed_replayer.getServerTime(callback);
        }
    }
    searchSymbols(userInput: string, exchange: string, symbolType: string, onResult: SearchSymbolsCallback): void {
        if (this.state == State.Origin) {
            return this.datafeed_original.searchSymbols(userInput, exchange, symbolType, onResult);
        } else {
            return this.datafeed_replayer.searchSymbols(userInput, exchange, symbolType, onResult);
        }
    }
    resolveSymbol(
        symbolName: string,
        onResolve: ResolveCallback,
        onError: DatafeedErrorCallback,
        extension?: SymbolResolveExtension | undefined
    ): void {
        if (this.state == State.Origin) {
            return this.datafeed_original.resolveSymbol(symbolName, onResolve, onError, extension);
        } else {
            return this.datafeed_replayer.resolveSymbol(symbolName, onResolve, onError, extension);
        }
    }
    getBars(
        symbolInfo: LibrarySymbolInfo,
        resolution: ResolutionString,
        periodParams: PeriodParams,
        onResult: HistoryCallback,
        onError: DatafeedErrorCallback
    ): void {
        if (this.state == State.Origin) {
            return this.datafeed_original.getBars(symbolInfo, resolution, periodParams, onResult, onError);
        } else {
            if (this.replay_time != null) {
                periodParams.to = Math.min(periodParams.to, this.replay_time);
            }
            if (periodParams.from > periodParams.to) {
                onResult([], { noData: true, nextTime: this.replay_time });
                return;
            }
            return this.datafeed_replayer.getBars(symbolInfo, resolution, periodParams, onResult, onError);
        }
    }
    subscribeBars(
        symbolInfo: LibrarySymbolInfo,
        resolution: ResolutionString,
        onTick: SubscribeBarsCallback,
        listenerGuid: string,
        onResetCacheNeededCallback: () => void
    ): void {
        if (this.state == State.Origin) {
            this.listenerSet.add(listenerGuid);

            return this.datafeed_original.subscribeBars(
                symbolInfo,
                resolution,
                onTick,
                listenerGuid,
                onResetCacheNeededCallback
            );
        } else {
            return this.datafeed_replayer.subscribeBars(
                symbolInfo,
                resolution,
                onTick,
                listenerGuid,
                onResetCacheNeededCallback
            );
        }
    }
    unsubscribeBars(listenerGuid: string): void {
        if (this.state == State.Origin) {
            return this.datafeed_original.unsubscribeBars(listenerGuid);
        } else {
            return this.datafeed_replayer.unsubscribeBars(listenerGuid);
        }
    }
    subscribeDepth?(symbol: string, callback: DOMCallback): string {
        throw new Error("Method not implemented.");
    }
    unsubscribeDepth?(subscriberUID: string): void {
        throw new Error("Method not implemented.");
    }
    getVolumeProfileResolutionForPeriod?(
        currentResolution: ResolutionString,
        from: number,
        to: number,
        symbolInfo: LibrarySymbolInfo
    ): ResolutionString {
        throw new Error("Method not implemented.");
    }
}
