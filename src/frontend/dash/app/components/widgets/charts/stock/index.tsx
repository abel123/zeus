"use client";

import {
    ChartingLibraryWidgetOptions,
    LanguageCode,
    ResolutionString,
} from "@/public/static/charting_library/charting_library";
import { useMemo } from "react";
import { TVChartContainer } from "../tv_chart";
import { ModelView } from "./models/model_view";
import { MacdConfig } from "./models/zen";
import { symbolState } from "@/app/store/dashboard";
import { useRecoilState } from "recoil";

const getLanguageFromURL = (): LanguageCode | null => {
    if (typeof window != "undefined") {
        const regex = new RegExp("[\\?&]lang=([^&#]*)");
        const results = regex.exec(location.search);
        return results === null ? null : (decodeURIComponent(results[1].replace(/\+/g, " ")) as LanguageCode);
    }

    return "en" as LanguageCode;
};

const defaultWidgetProps: Partial<ChartingLibraryWidgetOptions> = {
    library_path: "/static/charting_library/",
    locale: getLanguageFromURL() || "en",
    charts_storage_url: "https://saveload.tradingview.com",
    charts_storage_api_version: "1.1",
    client_id: "zen_user",
    user_id: "zen_user_id",
    fullscreen: false,
    autosize: true,
    symbol: "TSLA",
};

interface ChartConfig {
    resolution: ResolutionString;
    macd_config: MacdConfig[];
    hidden_extra_toolbar: boolean;
    standalone: boolean;
}

export const StockChart = (props: ChartConfig) => {
    const [symbol, setSymbol] = useRecoilState(symbolState);

    let tv = useMemo(() => {
        let mv = new ModelView(props.macd_config);
        mv.hidden_extra_toolbar = props.hidden_extra_toolbar;

        return (
            <TVChartContainer
                {...{
                    ...defaultWidgetProps,
                    interval: props.resolution,
                    model_view: mv,
                    standalone: props.standalone,
                }}
            />
        );
    }, [
        props.standalone ? "" : symbol,
        props.hidden_extra_toolbar,
        props.macd_config,
        props.resolution,
        props.standalone,
    ]);

    return <>{tv}</>;
};
