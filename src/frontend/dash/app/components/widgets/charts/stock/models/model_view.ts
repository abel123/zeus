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

export class ModelView {
    chart?: IChartWidgetApi;
    groupIds: ShapesGroupId[] = [];
    macd_indicator_id: (EntityId | null)[] = [];
    note_to_group = new Map<EntityId, ShapesGroupId>();
    beichi_to_note = new Map<string, EntityId>();
    line_to_beichi = new Map<EntityId, Beichi>();
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

        this.chart.createStudy("Volume", true);
        this.chart.createStudy(
            "Moving Average Multiple",
            false,
            false,
            {
                firstPeriods: 5,
                secondPeriods: 10,
                thirdPeriods: 20,
                fourthPeriods: 30,
                fifthPeriods: 60,
                sixthPeriods: 120,
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
        }
    }

    debounced_draw_zen = debounce(async () => {
        this.draw_zen();
    }, 400);

    draw_zen() {
        if (this.chart == undefined) {
            return;
        }
        let chart = this.chart;
        let range = chart.getVisibleRange();
        let symbol = chart.symbolExt();
        console.log("symbol", symbol);
        let resolution = chart.resolution();
        if (range.from >= range.to) {
            return;
        }

        axios
            .post<Zen>("http://127.0.0.1:8000/zen/elements", {
                from: range.from,
                to: range.to,
                symbol: symbol?.full_name,
                resolution: resolution,
                macd_config: this.macd_config,
            })
            .then((response) => {
                this.groupIds.forEach((id: any, idx: any) => {
                    try {
                        chart.shapesGroupController().removeGroup(id);
                    } catch (error) {}
                });
                this.groupIds = [];
                this.line_to_beichi.clear();

                console.log("response", response.data);

                chart.selection().clear();
                response.data.bi.finished.forEach((bi: BiInfo) => {
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
                        let bc_id = chart.createMultipointShape(
                            [
                                { price: bc.macd_a_val, time: bc.macd_a_dt },
                                { price: bc.macd_b_val, time: bc.macd_b_dt },
                            ],
                            {
                                shape: "arrow",
                                lock: true,
                                text: JSON.stringify({ type: "beichi", data: bc }),
                                overrides: {
                                    //linestyle: 1,
                                    linewidth: 2,
                                    linecolor: bc.direction == "down" ? "#ff1493" : "#00ce09",
                                },
                                ownerStudyId: this.macd_indicator_id[index] ?? undefined,
                                zOrder: "top",
                            }
                        );
                        if (bc_id) {
                            this.line_to_beichi.set(bc_id, bc);
                            chart.selection().add(bc_id);
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

    callback = (line_id: EntityId, event: DrawingEventType) => {
        if (this.chart == undefined || this.macd_indicator_id.length == 0) {
            return;
        }
        let chart = this.chart;
        if (event == "remove") {
            let group_id = this.note_to_group.get(line_id);
            //console.log("line", line_id, "group", group_id);

            if (group_id !== undefined) {
                chart.shapesGroupController().removeGroup(group_id);
                this.note_to_group.delete(line_id);
                Array.from(this.beichi_to_note.entries())
                    .filter((kv) => kv[1] == line_id)
                    .map((kv) => {
                        console.log("remove beichi line mapping", kv[0]);
                        this.beichi_to_note.delete(kv[0]);
                    });
            }
            return;
        }
        if (event != "click") {
            return;
        }
        console.log("id ", line_id, " ", event);

        let bc = this.line_to_beichi.get(line_id);
        if (bc === undefined || this.beichi_to_note.get(JSON.stringify([bc.start, bc.end])) !== undefined) {
            return;
        }

        try {
            console.log(
                "id ",
                line_id,
                " ",
                event,
                " bc ",
                JSON.stringify([bc.start, bc.end]),
                this.beichi_to_note.get(JSON.stringify([bc.start, bc.end]))?.toString()
            );
            chart.selection().clear();
            [bc?.start.left_dt, bc?.start.right_dt, bc?.end.left_dt, bc?.end.right_dt].forEach((dt) => {
                let id = chart.createShape(
                    { time: dt },
                    {
                        shape: "vertical_line",
                        overrides: {
                            linestyle: 1,
                            linewidth: 2,
                            linecolor: bc?.direction == "down" ? "#ff1493" : "#00ce09",
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
                        time: (bc?.start.right_dt + bc.end.left_dt) / 2,
                        price: bc.low,
                    },
                ],
                {
                    shape: "note",
                    text: ["macd_area", bc.macd_a_val.toFixed(2), bc.macd_b_val.toFixed(2)].join(" | "),
                }
            );

            let groupID = chart.shapesGroupController().createGroupFromSelection();
            chart.shapesGroupController().setGroupName(groupID, "beichi " + line_id);

            if (note_id) {
                this.note_to_group.set(note_id, groupID);
                console.log("line", line_id, "note", note_id, "group", groupID, "bc", [bc.start, bc.end]);

                if (bc != undefined) this.beichi_to_note.set(JSON.stringify([bc.start, bc.end]), note_id);
            }
        } catch (error) {
            return;
        }

        chart.selection().clear();
    };
}
