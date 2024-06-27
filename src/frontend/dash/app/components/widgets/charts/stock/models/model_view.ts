import {
    DrawingEventType,
    EntityId,
    IChartWidgetApi,
    ShapesGroupId,
    VisibleTimeRange,
} from "@/public/static/charting_library/charting_library";
import axios from "axios";
import { Beichi, Bi, BiInfo, End, MacdConfig, Start, Zen } from "./zen";
import { debounce } from "lodash";
import { DataFeedWrapper, State } from "../../tv_chart/datafeed";

export class ModelView {
    chart?: IChartWidgetApi;
    groupIds: ShapesGroupId[] = [];
    macd_indicator_id: (EntityId | null)[] = [];
    note_to_group = new Map<EntityId, ShapesGroupId>();
    beichi_to_note = new Map<string, EntityId>();
    macd_config: MacdConfig[] = [];
    hidden_extra_toolbar: boolean = false;
    constructor(macd_config: MacdConfig[]) {
        this.macd_config = macd_config;
    }

    async createMACD() {
        if (this.chart == undefined) {
            return;
        }
        let promizes = [];
        for (var config of this.macd_config) {
            let extras = {};
            if (config.source == "volume") {
                extras = {
                    "palettes.palette_0.colors.0.color": "rgba(0, 0, 0, 0.65)",
                    "palettes.palette_0.colors.1.color": "rgba(0, 0, 0, 0.3)",
                    "palettes.palette_0.colors.2.color": "rgba(152, 6, 101, 0.3)",
                    "palettes.palette_0.colors.3.color": "rgba(152, 6, 101, 0.7)",
                };
            }
            promizes.push(
                this.chart.createStudy(
                    config.source == "volume" ? "MACD" : "MACD-XD",
                    false,
                    false,
                    {
                        in_0: config.fast,
                        in_1: config.slow,
                        in_3: config.source ?? "close",
                        in_2: config.signal,
                    },
                    {
                        ...{
                            showLabelsOnPriceScale: false,
                            showLegendValues: true,
                        },
                        ...extras,
                    }
                )
            );
        }
        this.macd_indicator_id = await Promise.all(promizes);
    }

    async attach(api: IChartWidgetApi) {
        let self = this;
        this.chart = api;

        //this.chart.createStudy("Volume", true);
        this.chart.createStudy(
            "Moving Average Multiple",
            false,
            false,
            {
                firstPeriods: 5,
                secondPeriods: 15,
                thirdPeriods: 30,
                fourthPeriods: 60,
                fifthPeriods: 120,
                sixthPeriods: 200,
                method: "Simple",
            },
            {
                showLabelsOnPriceScale: false,
                showLegendValues: true,
            }
        );
        await this.createMACD();
        if (true || this.macd_indicator_id.length > 0) {
            api.onDataLoaded().subscribe(
                null,
                () => {
                    console.log("on data loaded");
                    self.debounced_draw_zen();
                },
                true
            );
            api.onVisibleRangeChanged().subscribe(this, function (visible_range: VisibleTimeRange) {
                console.log("range change");
                self.debounced_draw_zen();
            });
            setTimeout(() => {
                self.debounced_draw_zen();
            }, 1000);
            setTimeout(() => {
                self.debounced_draw_zen();
            }, 3000);
            setTimeout(() => {
                self.debounced_draw_zen();
            }, 5000);
        }
    }

    debounced_draw_zen = debounce(
        async () => {
            this.draw_zen();
        },
        800,
        { maxWait: 3000 }
    );

    draw_zen() {
        let update = (globalThis as any).zenUpdate ?? true;
        if (!update || this.chart == undefined) {
            return;
        }
        let Datafeed = (globalThis as any).datafeed as DataFeedWrapper;

        let chart = this.chart;
        let range = chart.getVisibleRange();
        let symbol = chart.symbolExt();
        console.log("symbol", symbol);
        let resolution = chart.resolution();
        if (range.from >= range.to) {
            return;
        }

        let headers = {};
        if ((globalThis as any).use_local) {
            headers = { Realtime: "false" };
        }
        axios
            .post<Zen>(
                "http://192.168.31.180:8080/zen/elements",
                {
                    from: range.from,
                    to: range.to,
                    symbol: symbol?.full_name,
                    resolution: resolution,
                    macd_config: this.macd_config,
                },
                {
                    headers: headers,
                }
            )
            .then((response) => {
                this.groupIds.forEach((id: any, idx: any) => {
                    try {
                        chart.shapesGroupController().removeGroup(id);
                    } catch (error) {}
                });
                this.groupIds = [];

                console.log("response", response.data);

                chart.selection().clear();
                response.data.bi.finished.forEach((bi: BiInfo) => {
                    if (bi.end_ts < range.from) {
                        return;
                    }
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
                    chart.shapesGroupController().setGroupName(group_id, "bi_finished");
                    this.groupIds.push(group_id);
                }
                chart.selection().clear();

                response.data.bi.unfinished.map((bi: BiInfo) => {
                    if (bi.start_ts >= bi.end_ts) {
                        return;
                    }
                    let bi_id = chart.createMultipointShape(
                        [
                            { price: bi.start, time: bi.start_ts },
                            { price: bi.end, time: bi.end_ts },
                        ],
                        {
                            shape: "trend_line",
                            //disableSelection: true,
                            lock: true,
                            overrides: {
                                linewidth: 2,
                                //linestyle: 1,
                                linecolor: "rgba(0, 0, 0, 0.6)",
                            },
                            //zOrder: "top",
                        }
                    );
                    if (bi_id) chart.selection().add(bi_id);
                });
                if (!chart.selection().isEmpty()) {
                    let group_id = chart.shapesGroupController().createGroupFromSelection();
                    chart.shapesGroupController().setGroupName(group_id, "bi_unfinished");
                    this.groupIds.push(group_id);
                }
                chart.selection().clear();

                response.data.beichi.forEach((bc_list: Beichi[], index: number) => {
                    bc_list.forEach((bc: Beichi) => {
                        if (bc.macd_b_dt < range.from) {
                            return;
                        }
                        let color = bc.direction == "down" ? "rgba(255, 20, 147, 1)" : "rgba(0, 206, 9, 1)";
                        if (bc.bc_type.indexOf("Diff") == -1) {
                            color = bc.direction == "down" ? "rgba(255, 20, 147, 1)" : "rgba(0, 206, 9, 1)";
                        }
                        if (bc.zs2.bi_count == 1) {
                            color = bc.direction == "down" ? "rgba(72, 138, 161, 0.6)" : "rgba(182, 138, 161, 1)";
                        } else {
                            let bc_id = chart.createMultipointShape(
                                [
                                    { price: bc.zs2.high, time: bc.zs2.left },
                                    { price: bc.zs2.low, time: bc.zs2.right },
                                ],
                                {
                                    shape: "rectangle",
                                    overrides: {
                                        backgroundColor: "rgba(200,200,200,0.15)",
                                        color: "rgba(100,100,100,0.8)",
                                        linewidth: 1.5,
                                    },
                                }
                            );
                            if (bc_id) chart.selection().add(bc_id);
                        }
                        let bc_id = chart.createMultipointShape(
                            [
                                { price: bc.macd_a_val, time: bc.macd_a_dt },
                                { price: bc.macd_b_val, time: bc.macd_b_dt },
                            ],
                            {
                                shape: "arrow",
                                lock: true,
                                overrides: {
                                    linestyle: 0,
                                    linewidth: bc.type.indexOf("Area") != -1 ? 1.5 : 2.3,
                                    linecolor: color,
                                },
                                ownerStudyId: this.macd_indicator_id[index] ?? undefined,
                                zOrder: "top",
                            }
                        );
                        if (bc_id) {
                            chart.selection().add(bc_id);
                        }

                        if (["FirstBuy", "SecondBuy", "ThirdBuy"].indexOf(bc.type) != -1) {
                            let bc_id = chart.createShape(
                                { price: bc.price, time: bc.dt },
                                {
                                    shape: "arrow_up",
                                    overrides: {
                                        arrowColor: "rgba(253, 45, 0, 1)",
                                    },
                                    zOrder: "bottom",
                                }
                            );
                            if (bc_id) chart.selection().add(bc_id);
                        } else if (bc.type != "None") {
                            let bc_id = chart.createShape(
                                { price: bc.price * 1.0, time: bc.dt },
                                {
                                    shape: "arrow_down",
                                    overrides: {
                                        arrowColor: "rgba(0, 206, 9, 1)",
                                    },
                                    zOrder: "bottom",
                                }
                            );
                            if (bc_id) chart.selection().add(bc_id);
                        }
                    });
                });
                if (!chart.selection().isEmpty()) {
                    let group_id = chart.shapesGroupController().createGroupFromSelection();
                    chart.shapesGroupController().setGroupName(group_id, "bi_beichi");
                    this.groupIds.push(group_id);
                    chart.selection().clear();
                }

                chart.selection().clear();

                response.data.bar_beichi.forEach((bars, index) => {
                    bars.forEach((ts) => {
                        let id = chart.createMultipointShape(
                            [
                                { time: ts, price: 0 },
                                { time: ts, price: -1 },
                            ],
                            {
                                shape: "ray",
                                overrides: {
                                    linestyle: 1,
                                    linewidth: 2,
                                    linecolor: "#999999",
                                    showTime: false,
                                },
                                ownerStudyId: this.macd_indicator_id[index] ?? undefined,
                                zOrder: "bottom",
                            }
                        );
                        if (id) chart.selection().add(id);
                    });
                });
                if (!chart.selection().isEmpty()) {
                    let group_id = chart.shapesGroupController().createGroupFromSelection();
                    chart.shapesGroupController().setGroupName(group_id, "bar_beichi");
                    this.groupIds.push(group_id);
                    chart.selection().clear();
                }
                chart.selection().clear();
            })
            .catch(function (error) {
                console.log(error);
            })
            .finally(function () {
                // always executed
            });
    }

    callback = (line_id: EntityId, event: DrawingEventType) => {};
}
