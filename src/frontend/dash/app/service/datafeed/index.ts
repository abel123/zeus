import type {
    Bar,
    HistoryCallback,
    IBasicDataFeed,
    LibrarySymbolInfo,
    OnReadyCallback,
    PeriodParams,
    ResolutionString,
    SearchSymbolResultItem,
    SearchSymbolsCallback,
    SubscribeBarsCallback,
    DatafeedConfiguration
} from "@/public/static/charting_library";
import axios from 'axios';


const supportedResolutions = [
    '1',
    '3',
    '5',
    '10',
    '15',
    '30',
    '60',
    '240',
    '1D',
    '1W',
    '1M'
] as ResolutionString[];

const config : DatafeedConfiguration = {
    supported_resolutions: supportedResolutions,
    exchanges: [
        { name: 'US', value: 'america', desc: "US market"},
        { name: 'HK', value: 'hongkong', desc: "Hongkong market"},
        { name: 'CN', value: 'china', desc:"China A stock market"},
    ],
};

type NetworkId = string;

const DataFeedFactory = (
    url: string
): IBasicDataFeed => {
    return {
        onReady: (cb: OnReadyCallback) => {
            setTimeout(() => cb(config), 500)
        },

        resolveSymbol: (symbolName: string, onSymbolResolvedCallback: (val: any) => any) => {
            console.log("resolving symbol", symbolName)
            axios.get<LibrarySymbolInfo>(url+"/resolve_symbol", {"params":{
                symbol: symbolName,
            }}).then(function (response) {
                console.log(response);
                onSymbolResolvedCallback(response.data);
              })
              .catch(function (error) {
                console.log(error);
              })
              .finally(function () {
                // always executed
              }); 
        },

        getBars: function (
            symbolInfo: LibrarySymbolInfo,
            resolution: ResolutionString,
            periodParams: PeriodParams,
            onHistoryCallback: HistoryCallback,
            onErrorCallback: (error: any) => any
        ) {
            console.log("getBars ", symbolInfo, resolution, periodParams);
            axios.post<Bar[]>(url+"/get_bars", {
                "symbol_info": symbolInfo,
                "resolution": resolution,
                "period_params": periodParams,
            }).then((response)=>{
                console.log(response);
                if(response.data.length <= 0){
                    onHistoryCallback(response.data, {"noData": true})
                } else {
                    onHistoryCallback(response.data)
                }
            })
            .catch(function (error) {
                console.log(error);
              })
              .finally(function () {
                // always executed
              }); 
        },

        subscribeBars: (
            symbolInfo: LibrarySymbolInfo,
            _resolution: ResolutionString,
            onTick: SubscribeBarsCallback
        ) => {
            console.log("subscribeBars ", symbolInfo)

        },
        
        unsubscribeBars: () => { 
            console.log("unsubscribeBars")
        },
        
        searchSymbols: (
            userInput: string,
            exchange: string,
            symbolType: string,
            onResult: SearchSymbolsCallback
        ) => {
            axios.get<SearchSymbolResultItem[]>(url+"/search_symbol", {"params":{
                user_input: userInput,
                exchange: exchange,
                type: symbolType,
            }}).then(function (response) {
                console.log(response);
                onResult(response.data);
              })
              .catch(function (error) {
                console.log(error);
              })
              .finally(function () {
                // always executed
              }); 
        },

    } as IBasicDataFeed;
}

export default DataFeedFactory;
