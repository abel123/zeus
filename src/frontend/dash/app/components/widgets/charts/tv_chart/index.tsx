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
    ChartingLibraryFeatureset,
    PineJS,
    IDropdownApi,
} from "@/public/static/charting_library";
import React from "react";
import { UDFCompatibleDatafeed } from "@/public/static/datafeeds/udf/src/udf-compatible-datafeed";
import { ModelView } from "../stock/models/model_view";
import { macd_xd } from "./custom_indicator";
import { useRecoilState } from "recoil";
import { replayState, symbolState } from "@/app/store/dashboard";
import { DataFeedWrapper } from "./datafeed";
import { LocalStorageDrawingsPerSymbolSaveLoadAdapter } from "./adapter";

export const TVChartContainer = (
    props: Partial<ChartingLibraryWidgetOptions> & { model_view: ModelView; standalone: boolean }
) => {
    const chartContainerRef = useRef<HTMLDivElement>() as React.MutableRefObject<HTMLInputElement>;
    const [symbol, setSymbol] = useRecoilState(symbolState);
    const [replay, setReplay] = useRecoilState(replayState);

    let Datafeed = (globalThis as any).datafeed as DataFeedWrapper;
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
            datafeed: Datafeed,
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
            enabled_features: [
                "hide_left_toolbar_by_default",
                "pre_post_market_sessions",
                "saveload_separate_drawings_storage",
            ],
            charts_storage_url: props.charts_storage_url,
            charts_storage_api_version: props.charts_storage_api_version,
            client_id: props.client_id,
            user_id: props.user_id,
            fullscreen: props.fullscreen,
            autosize: props.autosize,
            symbol_search_request_delay: 6000,
            save_load_adapter: new LocalStorageDrawingsPerSymbolSaveLoadAdapter(),
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

            tvWidget.headerReady().then(async () => {
                // BAR REPLAY

                var bar_replay_status = 0;
                var bar_replay = tvWidget.createButton();
                bar_replay.setAttribute("title", "Bar replay");

                bar_replay.addEventListener("click", () => {
                    if (bar_replay_status == 0) {
                        bar_replay.innerHTML =
                            '<div data-role="button" class="button" style="padding:2px"; display: inline-flex; align-items: center;"> <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 28 28" width="28" height="28"> <path fill="none" stroke="blue" d="M13.5 20V9l-6 5.5 6 5.5zM21.5 20V9l-6 5.5 6 5.5z"></path> </svg></div>';
                        tvWidget
                            .activeChart()
                            .requestSelectBar()
                            .then((time) => {
                                Datafeed.set_replay_state(time);
                                setReplay(time);
                            });
                    } else {
                        bar_replay.innerHTML =
                            '<div data-role="button" class="button" style="padding:2px"; display: inline-flex; align-items: center;"> <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 28 28" width="28" height="28"> <path fill="none" stroke="currentColor" d="M13.5 20V9l-6 5.5 6 5.5zM21.5 20V9l-6 5.5 6 5.5z"></path> </svg></div>';
                        tvWidget.activeChart().cancelSelectBar();
                    }

                    bar_replay_status = bar_replay_status == 0 ? 1 : 0;
                });

                bar_replay.innerHTML =
                    '<div data-role="button" class="button" style="padding:2px"; display: inline-flex; align-items: center;"> <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 28 28" width="28" height="28"> <path fill="none" stroke="currentColor" d="M13.5 20V9l-6 5.5 6 5.5zM21.5 20V9l-6 5.5 6 5.5z"></path> </svg></div>';

                // PLAY
                var bar_replay_controls_status = 0;
                var bar_replay_controls = tvWidget.createButton();
                bar_replay_controls.setAttribute("title", "Play");

                bar_replay_controls.addEventListener("click", () => {
                    if (bar_replay_controls_status == 0) {
                        Datafeed.bar_replay_start();
                        bar_replay_controls.innerHTML =
                            '<div data-role="button" class="button" style="padding:2px"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 28 28" width="28" height="28"><path fill="currentColor" fill-rule="evenodd" d="M10 6h2v16h-2V6ZM9 6a1 1 0 0 1 1-1h2a1 1 0 0 1 1 1v16a1 1 0 0 1-1 1h-2a1 1 0 0 1-1-1V6Zm7 0h2v16h-2V6Zm-1 0a1 1 0 0 1 1-1h2a1 1 0 0 1 1 1v16a1 1 0 0 1-1 1h-2a1 1 0 0 1-1-1V6Z"></path></svg></div>';
                    } else {
                        Datafeed.bar_replay_stop();
                        bar_replay_controls.innerHTML =
                            '<div data-role="button" class="button" style="padding:2px"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 28 28" width="28" height="28"><path fill="currentColor" fill-rule="evenodd" d="m10.997 6.93 7.834 6.628a.58.58 0 0 1 0 .88l-7.834 6.627c-.359.303-.897.04-.897-.44V7.37c0-.48.538-.743.897-.44Zm8.53 5.749a1.741 1.741 0 0 1 0 2.637l-7.834 6.628c-1.076.91-2.692.119-2.692-1.319V7.37c0-1.438 1.616-2.23 2.692-1.319l7.834 6.628Z"></path></svg></div>';
                    }

                    bar_replay_controls_status = bar_replay_controls_status == 0 ? 1 : 0;
                });

                bar_replay_controls.innerHTML =
                    '<div data-role="button" class="button" style="padding:2px"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 28 28" width="28" height="28"><path fill="currentColor" fill-rule="evenodd" d="m10.997 6.93 7.834 6.628a.58.58 0 0 1 0 .88l-7.834 6.627c-.359.303-.897.04-.897-.44V7.37c0-.48.538-.743.897-.44Zm8.53 5.749a1.741 1.741 0 0 1 0 2.637l-7.834 6.628c-1.076.91-2.692.119-2.692-1.319V7.37c0-1.438 1.616-2.23 2.692-1.319l7.834 6.628Z"></path></svg></div>';

                // FORWARD (STEP)

                var bar_replay_step = tvWidget.createButton();
                bar_replay_step.setAttribute("title", "Forward");

                bar_replay_step.addEventListener("click", () => {
                    Datafeed.bar_replay_step();
                });

                bar_replay_step.innerHTML =
                    '<div data-role="button" class="button" style="padding:2px"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 28 28" width="28" height="28"><path fill="currentColor" fill-rule="evenodd" d="M20 6v16h1V6h-1Zm-3.908 7.628L9.834 7.996A.5.5 0 0 0 9 8.368v11.264a.5.5 0 0 0 .834.372l6.258-5.632a.5.5 0 0 0 0-.744Zm.67 1.487a1.5 1.5 0 0 0 0-2.23l-6.259-5.632C9.538 6.384 8 7.07 8 8.368v11.264c0 1.299 1.538 1.984 2.503 1.115l6.258-5.632Z"></path></svg></div>';

                // DROPDOWN

                let dropdown: IDropdownApi;

                const change_title = (new_title: string) => {
                    dropdown.applyOptions({
                        title: new_title,
                    });
                };

                dropdown = await tvWidget.createDropdown({
                    title: "1x",
                    tooltip: "Replay speed",
                    items: [
                        {
                            title: "10x",
                            onSelect: () => {
                                change_title("10x");
                                Datafeed.bar_replay_set_speed("100");
                            },
                        },
                        {
                            title: "5x",
                            onSelect: () => {
                                change_title("5x");
                                Datafeed.bar_replay_set_speed("500");
                            },
                        },
                        {
                            title: "3x",
                            onSelect: () => {
                                change_title("3x");
                                Datafeed.bar_replay_set_speed("300");
                            },
                        },
                        {
                            title: "1x",
                            onSelect: () => {
                                change_title("1x");
                                Datafeed.bar_replay_set_speed("1000");
                            },
                        },
                        {
                            title: "0.5x",
                            onSelect: () => {
                                change_title("0.5x");
                                Datafeed.bar_replay_set_speed("2000");
                            },
                        },
                        {
                            title: "0.3x",
                            onSelect: () => {
                                change_title("0.3x");
                                Datafeed.bar_replay_set_speed("3000");
                            },
                        },
                        {
                            title: "0.1x",
                            onSelect: () => {
                                change_title("0.1x");
                                Datafeed.bar_replay_set_speed("10000");
                            },
                        },
                    ],
                });
            });
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
        replay,
    ]);

    return (
        <>
            <div ref={chartContainerRef} className={styles.TVChartContainer} />
        </>
    );
};
