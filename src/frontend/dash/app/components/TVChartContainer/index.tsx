"use client";

import styles from "./index.module.css";
import { useEffect, useRef } from "react";
import {
    ChartingLibraryWidgetOptions,
    LanguageCode,
    ResolutionString,
    widget,
    Bar,
    EntityId,
    DrawingEventType,
    ShapesGroupId,
} from "@/public/static/charting_library";
import DataFeedFactory from "./lib/datafeed";
import { url } from "inspector";
import axios from "axios";
import { debounce } from "lodash";
import React from "react";
import { UDFCompatibleDatafeed } from "@/public/static/datafeeds/udf/src/udf-compatible-datafeed";

export const TVChartContainer = (props: Partial<ChartingLibraryWidgetOptions>) => {
    const chartContainerRef = useRef<HTMLDivElement>() as React.MutableRefObject<HTMLInputElement>;

    useEffect(() => {
        const widgetOptions: ChartingLibraryWidgetOptions = {
            symbol: props.symbol,
            // BEWARE: no trailing slash is expected in feed URL
            //datafeed: (globalThis as any).datafeed ?? DataFeedFactory("http://127.0.0.1:8000"),
            datafeed: new UDFCompatibleDatafeed("http://127.0.0.1:8000/datafeed/udf", 16 * 1000),
            interval: props.interval as ResolutionString,
            container: chartContainerRef.current,
            library_path: props.library_path,
            locale: props.locale as LanguageCode,
            disabled_features: ["use_localstorage_for_settings"],
            enabled_features: ["study_templates"],
            charts_storage_url: props.charts_storage_url,
            charts_storage_api_version: props.charts_storage_api_version,
            client_id: props.client_id,
            user_id: props.user_id,
            fullscreen: props.fullscreen,
            autosize: props.autosize,
            symbol_search_request_delay: 1200,
            //theme: "Dark",
            overrides: {
                "mainSeriesProperties.candleStyle.borderDownColor": "#089981",
                "mainSeriesProperties.candleStyle.borderUpColor": "#F23645",
                "mainSeriesProperties.candleStyle.downColor": "#089981",
                "mainSeriesProperties.candleStyle.upColor": "#F23645",
                "mainSeriesProperties.candleStyle.wickDownColor": "#089981",
                "mainSeriesProperties.candleStyle.wickUpColor": "#F23645",
            },
        };
        const tvWidget = new widget(widgetOptions);
        // @ts-ignore
        window.tvWidget = tvWidget;

        tvWidget.onChartReady(() => {
            tvWidget.subscribe("chart_load_requested", (event) => {
                console.log("chart_load_requested", event);
            });

            tvWidget.headerReady().then(() => {
                const button = tvWidget.createButton();
                button.innerHTML = "Draw Zen";
                button.addEventListener("click", () => {});
            });

            let chart = tvWidget.activeChart();
            chart.createStudy("MACD").then((macd_indicator_id) => {
                function draw_zen() {
                    let range = chart.getVisibleRange();
                    let symbol = chart.symbolExt();
                    let resolution = chart.resolution();
                    if (range.from >= range.to) {
                        return;
                    }

                    axios
                        .get("http://127.0.0.1:8000/zen/elements", {
                            params: {
                                from: range.from,
                                to: range.to,
                                symbol: symbol?.symbol,
                                resolution: resolution,
                            },
                        })
                        .then((response) => {
                            //chart.removeAllShapes();
                            chart
                                .shapesGroupController()
                                .groups()
                                .forEach((id, idx) => {
                                    if (chart.shapesGroupController().getGroupName(id) == "bi_refresh")
                                        chart.shapesGroupController().removeGroup(id);
                                });
                            console.log("response", response.data);

                            chart.selection().clear();
                            response.data["bi"]["finished"].forEach((bi: any) => {
                                let bi_id = chart.createMultipointShape(
                                    [
                                        { price: bi.start, time: bi.start_ts },
                                        { price: bi.end, time: bi.end_ts },
                                    ],
                                    {
                                        shape: "trend_line",
                                        //disableSelection: true,
                                        showInObjectsTree: false,
                                        lock: true,
                                        text: "test",
                                    }
                                );
                                if (bi_id !== null) chart.selection().add(bi_id);
                            });
                            if (!chart.selection().isEmpty()) {
                                let group_id = chart.shapesGroupController().createGroupFromSelection();
                                chart.shapesGroupController().setGroupName(group_id, "bi_refresh");
                            }
                            chart.selection().clear();

                            response.data["bi"]["unfinished"].map((bi: any) => {
                                let bi_id = chart.createMultipointShape(
                                    [
                                        { price: bi.start, time: bi.start_ts },
                                        { price: bi.end, time: bi.end_ts },
                                    ],
                                    {
                                        shape: "trend_line",
                                        //disableSelection: true,
                                        text: "test",
                                        lock: true,
                                        overrides: {
                                            //linestyle: 1,
                                            linewidth: 1,
                                            linecolor: "#ff7373",
                                        },
                                    }
                                );
                                if (bi_id) chart.selection().add(bi_id);
                            });
                            if (!chart.selection().isEmpty()) {
                                let group_id = chart.shapesGroupController().createGroupFromSelection();
                                chart.shapesGroupController().setGroupName(group_id, "bi_refresh");
                            }
                            chart.selection().clear();

                            response.data["beichi"].map((bc: any) => {
                                let bc_id = chart.createMultipointShape(
                                    [
                                        { price: bc.macd_a_val, time: bc.macd_a_dt },
                                        { price: bc.macd_b_val, time: bc.macd_b_dt },
                                    ],
                                    {
                                        shape: "arrow",
                                        //disableSelection: true,
                                        lock: true,

                                        text: JSON.stringify({ type: "beichi", data: bc }),
                                        overrides: {
                                            //linestyle: 1,
                                            linewidth: 2,
                                            linecolor: bc.direction == "down" ? "#ff1493" : "#00ce09",
                                        },
                                        ownerStudyId: macd_indicator_id as EntityId,
                                        zOrder: "top",
                                    }
                                );
                                if (bc_id) chart.selection().add(bc_id);
                            });
                            if (!chart.selection().isEmpty()) {
                                let group_id = chart.shapesGroupController().createGroupFromSelection();
                                chart.shapesGroupController().setGroupName(group_id, "bi_refresh");
                                chart.selection().clear();
                            }
                        })
                        .catch(function (error) {
                            console.log(error);
                        })
                        .finally(function () {
                            // always executed
                        });
                }
                const debounced_draw_zen = debounce(async () => {
                    draw_zen();
                }, 100);
                chart.onDataLoaded().subscribe(
                    null,
                    () => {
                        console.log("on data loaded");
                        debounced_draw_zen();
                    },
                    true
                );
                tvWidget.subscribe("onTick", (tick: Bar) => {
                    console.log("on tick");
                    debounced_draw_zen();
                });
                chart.onVisibleRangeChanged().subscribe(null, function (visible_range) {
                    console.log("range change");
                    debounced_draw_zen();
                });

                let note_id_mapping = new Map<EntityId, ShapesGroupId>();
                let beichi_line_mapping = new Map<EntityId, EntityId>();
                tvWidget.subscribe("drawing_event", function (line_id: EntityId, event: DrawingEventType) {
                    if (event == "remove") {
                        let group_id = note_id_mapping.get(line_id);
                        if (group_id !== undefined) {
                            chart.shapesGroupController().removeGroup(group_id);
                            note_id_mapping.delete(line_id);
                            Array.from(beichi_line_mapping.entries())
                                .filter((kv) => kv[1] == line_id)
                                .map((kv) => {
                                    console.log("remove beichi line", kv[0]);
                                    beichi_line_mapping.delete(kv[0]);
                                });
                        }
                        return;
                    }
                    if (event != "click") {
                        return;
                    }
                    if (beichi_line_mapping.get(line_id) !== undefined) {
                        return;
                    }
                    let text = chart.getShapeById(line_id).getProperties()["text"];
                    if (text == "") {
                        return;
                    }
                    console.log("id ", line_id, " ", event, " ", text);

                    try {
                        let item = JSON.parse(text);
                        console.log("id ", line_id, " ", event, " ", text);

                        if (item["type"] != "beichi") {
                            return;
                        }

                        [
                            item["data"]["start"]["left_dt"],
                            item["data"]["start"]["right_dt"],
                            item["data"]["end"]["left_dt"],
                            item["data"]["end"]["right_dt"],
                        ].forEach((dt) => {
                            let id = chart.createShape(
                                { time: dt },
                                {
                                    shape: "vertical_line",
                                    overrides: {
                                        linestyle: 1,
                                        linewidth: 2,
                                        linecolor: item["data"]["direction"] == "down" ? "#ff1493" : "#00ce09",
                                        showTime: false,
                                    },
                                }
                            );
                            if (id) {
                                chart.selection().add(id);
                            }
                        });

                        let note_id = chart.createMultipointShape(
                            [
                                {
                                    time: (item["data"]["start"]["right_dt"] + item["data"]["end"]["left_dt"]) / 2,
                                    price: item["data"]["low"],
                                },
                            ],
                            {
                                shape: "note",
                                text: [
                                    "macd_area",
                                    item["data"]["macd_a_val"].toFixed(2),
                                    item["data"]["macd_b_val"].toFixed(2),
                                ].join(" | "),
                            }
                        );

                        let groupID = chart.shapesGroupController().createGroupFromSelection();
                        chart.shapesGroupController().setGroupName(groupID, "beichi");

                        if (note_id) {
                            chart.selection().add(note_id);
                            note_id_mapping.set(note_id, groupID);
                            beichi_line_mapping.set(line_id, note_id);
                        }
                    } catch (error) {
                        return;
                    }

                    chart.selection().clear();
                });
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
