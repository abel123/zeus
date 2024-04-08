import {
    CustomIndicator,
    RawStudyMetaInfoId,
    StudyInputType,
    StudyPlotType,
    PineJS,
    IContext,
} from "@/public/static/charting_library/charting_library";

export const macd_xd = (PineJS: PineJS) => {
    return {
        name: "MACD XD",
        metainfo: {
            _metainfoVersion: 53,

            id: "macd_xd@tv-basicstudies-1" as RawStudyMetaInfoId,
            name: "MACD-XD",
            description: "MACD-XD",
            shortDescription: "MACD-XD",

            isCustomIndicator: true,
            format: {
                type: "price",
                precision: 2,
            },
            is_hidden_study: false,
            is_price_study: false, // whether the study should appear on the main series pane.
            linkedToSeries: true, // whether the study price scale should be the same as the main series one.
            plots: [
                {
                    id: "hist",
                    type: "line" as StudyPlotType.Line,
                },
                {
                    id: "macd",
                    type: "line" as StudyPlotType.Line,
                    linewidth: 1,
                },
                {
                    id: "signal",
                    type: "line" as StudyPlotType.Line,
                    linewidth: 1,
                },
                {
                    id: "plot_hist_color",
                    type: "colorer",
                    palette: "palette_0",
                    target: "hist",
                },
            ],
            ohlcPlots: {},
            defaults: {
                styles: {
                    macd: {
                        color: "#2962FF",
                        linewidth: 1.2,
                    },
                    signal: {
                        color: "#FF6D00",
                        linewidth: 1.2,
                    },
                    hist: {
                        color: "#26A69A",
                        plottype: 5,
                        linestyle: 0,
                        visible: true,
                        linewidth: 1,
                    },
                },
                palettes: {
                    palette_0: {
                        colors: [
                            { color: "rgba(172, 229, 220, 1)", style: 0, width: 1 },
                            { color: "rgba(34, 171, 148, 1)", style: 0, width: 1 },
                            { color: "rgba(242, 54, 69, 1)", style: 0, width: 1 },
                            { color: "rgba(252, 203, 205, 1)", style: 0, width: 1 },
                            { color: "rgba(0, 0, 0, 0.3)", style: 0, width: 1 }, // above beichi
                            { color: "rgba(152, 6, 101, 0.3)", style: 0, width: 1 }, // below beichi
                        ],
                    },
                },
                inputs: {
                    in_0: 12,
                    in_1: 26,
                    in_3: "close",
                    in_2: 9,
                    sma_source: false,
                    sma_signal: false,
                },
                precision: 4,
            },
            palettes: {
                palette_0: {
                    colors: [
                        { name: "col_grow_below" },
                        { name: "col_fall_below" },
                        { name: "col_grow_above" },
                        { name: "col_fall_above" },
                        { name: "above beichi" },
                        { name: "below beichi" },
                    ],
                    valToIndex: {
                        0: 0,
                        1: 1,
                        2: 2,
                        3: 3,
                    },
                },
            },
            styles: {
                hist: {
                    title: "Hist",
                    text: "Hist",
                    histogramBase: 0,
                },
                macd: {
                    title: "macd",
                    text: "macd",
                },
                signal: {
                    title: "Signal",
                    text: "Signal",
                },
            },
            inputs: [
                { id: "in_0", name: "Fast Length", type: "integer" as StudyInputType.Integer, defval: 12 },
                { id: "in_1", name: "Slow Length", type: "integer" as StudyInputType.Integer, defval: 26 },
                {
                    id: "in_3",
                    name: "Source",
                    type: "source" as StudyInputType.Source,
                    defval: "close",
                    options: ["open", "high", "low", "close", "volume", "hl2", "hlc3", "ohlc4"],
                },
                {
                    id: "in_2",
                    name: "Signal Smoothing",
                    type: "integer" as StudyInputType.Integer,
                    min: 1,
                    max: 50,
                    defval: 9,
                },
                { id: "sma_source", name: "Simple MA(Oscillator)", type: "bool" as StudyInputType.Bool, defval: false },
                {
                    id: "sma_signal",
                    name: "Simple MA(Signal Line)",
                    type: "bool" as StudyInputType.Bool,
                    defval: false,
                },
            ],
        },
        constructor: function () {
            this.last_hist = -1;
            this.last_low = -1;
            this.last_high = 1;
            this.main = function (ctx: IContext, inputCallback) {
                let period = PineJS.Std.period(ctx);
                this._context = ctx;

                this._input = inputCallback;

                var fast_length = this._input(0);
                var slow_length = this._input(1);
                var src = this._input(2);
                var signal_length = this._input(3);
                var sma_source = this._input(4);
                var sma_signal = this._input(5);

                var source = undefined;
                if (src === "close") {
                    source = this._context.new_unlimited_var(PineJS.Std.close(this._context));
                } else if (src === "open") {
                    source = this._context.new_unlimited_var(PineJS.Std.open(this._context));
                } else if (src === "high") {
                    source = this._context.new_unlimited_var(PineJS.Std.high(this._context));
                } else if (src === "low") {
                    source = this._context.new_unlimited_var(PineJS.Std.low(this._context));
                } else if (src === "volume") {
                    source = this._context.new_unlimited_var(PineJS.Std.volume(this._context));
                } else if (src === "hl2") {
                    source = this._context.new_unlimited_var(PineJS.Std.hl2(this._context));
                } else if (src === "hlc3") {
                    source = this._context.new_unlimited_var(PineJS.Std.hlc3(this._context));
                } else if (src === "ohlc4") {
                    source = this._context.new_unlimited_var(PineJS.Std.ohlc4(this._context));
                }

                let low = PineJS.Std.low(this._context);
                let high = PineJS.Std.high(this._context);

                var fast_ma = sma_source
                    ? PineJS.Std.sma(source, fast_length, this._context)
                    : PineJS.Std.ema(source, fast_length, this._context);
                var slow_ma = sma_source
                    ? PineJS.Std.sma(source, slow_length, this._context)
                    : PineJS.Std.ema(source, slow_length, this._context);
                var macd = this._context.new_unlimited_var(fast_ma - slow_ma);
                var signal = sma_signal
                    ? PineJS.Std.sma(macd, signal_length, this._context)
                    : PineJS.Std.ema(macd, signal_length, this._context);
                var hist = 1 * (macd.get(0) - signal);

                let col_grow_below = 0;
                let col_fall_below = 1;
                let col_grow_above = 2;
                let col_fall_above = 3;
                let col_above_beichi = 4;
                let col_below_beichi = 5;
                let color = col_grow_above;
                if (hist > 0) {
                    if (this.last_hist < hist) {
                        color = col_grow_above;
                    } else {
                        color = col_fall_above;
                        if (period === "1" && high > this.last_high && src != "volume") {
                            color = col_above_beichi;
                        }
                    }
                } else {
                    // 柱子向上
                    if (this.last_hist < hist) {
                        color = col_grow_below;
                        if (period === "1" && src != "volume" && low < this.last_low) {
                            //价向下
                            color = col_below_beichi;
                        }
                    } else {
                        color = col_fall_below;
                    }
                }

                //console.log("params", fast_length, slow_length, src, signal_length, sma_source, sma_signal);
                //console.log("value", macd.get(0), signal, hist, this.last_hist, color);
                this.last_hist = hist;
                this.last_low = low;
                this.last_high = high;
                return [hist, macd.get(0), signal, color];
            };
        },
    } as CustomIndicator;
};
