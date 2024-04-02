"use client";

import styles from "./index.module.css";
import { useContext, useEffect, useRef } from "react";
import {
    ChartingLibraryWidgetOptions,
    LanguageCode,
    ResolutionString,
    widget as Widget,
    Bar,
    EntityId,
    DrawingEventType,
    ShapesGroupId,
    TradingTerminalWidgetOptions,
    CrossHairMovedEventParams,
    ChartingLibraryFeatureset,
    PineJS,
} from "@/public/static/charting_library";
import React from "react";
import { UDFCompatibleDatafeed } from "@/public/static/datafeeds/udf/src/udf-compatible-datafeed";
import { ModelView } from "../stock/models/model_view";
import { macd_xd } from "./custom_indicator";
import { useRecoilState } from "recoil";
import { symbolState } from "@/app/store/dashboard";

export const TVChartContainer = (
    props: Partial<ChartingLibraryWidgetOptions> & { model_view: ModelView; standalone: boolean }
) => {
    const chartContainerRef = useRef<HTMLDivElement>() as React.MutableRefObject<HTMLInputElement>;
    const [symbol, setSymbol] = useRecoilState(symbolState);

    useEffect(() => {
        let extras = {};
        if (props.interval == ("1" as ResolutionString)) {
            extras = {
                "paneProperties.background": "#FBFCFB",
                "paneProperties.backgroundType": "solid",
            };
        }
        const widgetOptions: ChartingLibraryWidgetOptions = {
            symbol: props.standalone ? props.symbol : symbol,
            // BEWARE: no trailing slash is expected in feed URL
            //datafeed: (globalThis as any).datafeed ?? DataFeedFactory("http://127.0.0.1:8000"),
            datafeed:
                (globalThis as any).datafeed ??
                new UDFCompatibleDatafeed("http://127.0.0.1:8080/datafeed/udf", 5 * 1000),
            interval: props.interval as ResolutionString,
            container: chartContainerRef.current,
            library_path: props.library_path,
            locale: props.locale as LanguageCode,
            disabled_features: [
                "use_localstorage_for_settings",
                //"trading_account_manager",
                //"keep_object_tree_widget_in_right_toolbar",
                //"right_toolbar",
                //"drawing_templates",
                /* // for reduce extra display space
                "hide_left_toolbar_by_default",
                "left_toolbar",
                "timeframes_toolbar",
                "header_widget",
                                */
                "widget_logo" as ChartingLibraryFeatureset,
            ],
            custom_indicators_getter: (pineJs: PineJS) => {
                return Promise.resolve([macd_xd(pineJs)]);
            },
            enabled_features: ["hide_left_toolbar_by_default", "pre_post_market_sessions"],
            charts_storage_url: props.charts_storage_url,
            charts_storage_api_version: props.charts_storage_api_version,
            client_id: props.client_id,
            user_id: props.user_id,
            fullscreen: props.fullscreen,
            autosize: props.autosize,
            symbol_search_request_delay: 6000,
            //theme: "Dark",
            timezone: "Asia/Chongqing",
            overrides: {
                ...{
                    "mainSeriesProperties.candleStyle.borderDownColor": "#089981",
                    "mainSeriesProperties.candleStyle.borderUpColor": "#F23645",
                    "mainSeriesProperties.candleStyle.downColor": "#089981",
                    "mainSeriesProperties.candleStyle.upColor": "#F23645",
                    "mainSeriesProperties.candleStyle.wickDownColor": "#089981",
                    "mainSeriesProperties.candleStyle.wickUpColor": "#F23645",
                },
                ...extras,
            },
            studies_overrides: {
                "MACD.palettes.palette_0.colors.0.color": "rgba(242, 54, 69, 1)",
                "MACD.palettes.palette_0.colors.1.color": "rgba(252, 203, 205, 1)",
                "MACD.palettes.palette_0.colors.2.color": "rgba(172, 229, 220, 1)",
                "MACD.palettes.palette_0.colors.3.color": "rgba(34, 171, 148, 1)",
            },
        };
        if (props.model_view.hidden_extra_toolbar) {
            widgetOptions.disabled_features?.push("left_toolbar", "timeframes_toolbar", "header_widget");
        }
        const tvWidget = new Widget(widgetOptions);
        // @ts-ignore
        window.tvWidget = tvWidget;
        tvWidget.onChartReady(() => {
            let chart = tvWidget.activeChart();
            props.model_view.attach(chart).then(() => {
                console.log("model_view attached");
            });

            tvWidget.subscribe("onTick", (tick: Bar) => {
                console.log("on tick");
                props.model_view.debounced_draw_zen();
            });

            tvWidget.subscribe("drawing_event", function (line_id: EntityId, event: DrawingEventType) {
                props.model_view.callback(line_id, event);
            });

            chart.onSymbolChanged().subscribe(
                null,
                () => {
                    //setSymbol(chart.symbol().split(" ")[0]);
                },
                false
            );
        });

        return () => {
            tvWidget.remove();
        };
    }, [
        props.autosize,
        props.charts_storage_api_version,
        props.charts_storage_url,
        props.client_id,
        props.fullscreen,
        props.interval,
        props.library_path,
        props.locale,
        props.model_view,
        props.standalone,
        props.symbol,
        props.user_id,
        props.standalone ? "" : symbol,
    ]);

    return (
        <>
            <div ref={chartContainerRef} className={styles.TVChartContainer} />
        </>
    );
};
