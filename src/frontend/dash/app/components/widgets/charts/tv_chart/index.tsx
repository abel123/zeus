"use client";

import styles from "./index.module.css";
import { useEffect, useRef } from "react";
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
} from "@/public/static/charting_library";
import axios from "axios";
import { debounce } from "lodash";
import React from "react";
import { UDFCompatibleDatafeed } from "@/public/static/datafeeds/udf/src/udf-compatible-datafeed";
import { ModelView } from "../stock/models/model_view";

export const TVChartContainer = (props: Partial<ChartingLibraryWidgetOptions> & { model_view: ModelView }) => {
    const chartContainerRef = useRef<HTMLDivElement>() as React.MutableRefObject<HTMLInputElement>;

    useEffect(() => {
        const widgetOptions: TradingTerminalWidgetOptions = {
            symbol: props.symbol,
            // BEWARE: no trailing slash is expected in feed URL
            //datafeed: (globalThis as any).datafeed ?? DataFeedFactory("http://127.0.0.1:8000"),
            datafeed: new UDFCompatibleDatafeed("http://127.0.0.1:8000/datafeed/udf", 5 * 1000),
            interval: props.interval as ResolutionString,
            container: chartContainerRef.current,
            library_path: props.library_path,
            locale: props.locale as LanguageCode,
            disabled_features: [
                "use_localstorage_for_settings",
                "trading_account_manager",
                //"keep_object_tree_widget_in_right_toolbar",
                //"right_toolbar",
                "drawing_templates",
                /* // for reduce extra display space
                "hide_left_toolbar_by_default",
                "left_toolbar",
                "timeframes_toolbar",
                */
            ],
            enabled_features: ["hide_left_toolbar_by_default"],
            charts_storage_url: props.charts_storage_url,
            charts_storage_api_version: props.charts_storage_api_version,
            client_id: props.client_id,
            user_id: props.user_id,
            fullscreen: props.fullscreen,
            autosize: props.autosize,
            symbol_search_request_delay: 1200,
            //theme: "Dark",
            timezone: "Asia/Chongqing",
            overrides: {
                "mainSeriesProperties.candleStyle.borderDownColor": "#089981",
                "mainSeriesProperties.candleStyle.borderUpColor": "#F23645",
                "mainSeriesProperties.candleStyle.downColor": "#089981",
                "mainSeriesProperties.candleStyle.upColor": "#F23645",
                "mainSeriesProperties.candleStyle.wickDownColor": "#089981",
                "mainSeriesProperties.candleStyle.wickUpColor": "#F23645",
            },
        };
        const tvWidget = new Widget(widgetOptions);
        // @ts-ignore
        window.tvWidget = tvWidget;
        tvWidget.onChartReady(() => {
            let chart = tvWidget.activeChart();
            chart.crossHairMoved().subscribe(
                null,
                (p: CrossHairMovedEventParams) => {
                    console.log("crosshair", p);
                },
                false
            );
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
        });

        return () => {
            tvWidget.remove();
        };
    }, [props]);

    return (
        <>
            <div ref={chartContainerRef} className={styles.TVChartContainer} />
        </>
    );
};
