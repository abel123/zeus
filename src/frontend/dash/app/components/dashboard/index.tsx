import React from "react";
import GridLayout from "react-grid-layout";
import { StockChart } from "../widgets/charts/stock";
import { ResolutionString } from "@/public/static/charting_library/charting_library";

export class MainGrid extends React.Component {
    render() {
        return (
            <div className="grid grid-cols-12 h-full">
                <div className="col-span-9 grid grid-cols-9 grid-rows-6">
                    <div className="row-span-2 col-span-5 border-solid border-2 border-slate-300" key="1-1">
                        <StockChart
                            default_symbol={"TSLA"}
                            resolution={"1" as ResolutionString}
                            macd_config={[
                                { fast: 4, slow: 9, signal: 9 },
                                { fast: 12, slow: 26, signal: 9 },
                            ]}
                            hidden_extra_toolbar={true}
                        />
                    </div>
                    <div className="row-span-3 col-span-4 border-solid border-2 border-slate-300" key="1-2">
                        <StockChart
                            default_symbol={"TSLA"}
                            resolution={"3" as ResolutionString}
                            macd_config={[
                                { fast: 4, slow: 9, signal: 9 },
                                { fast: 12, slow: 26, signal: 9 },
                            ]}
                            hidden_extra_toolbar={true}
                        />
                    </div>
                    <div className="row-span-4 col-span-5 border-solid border-2 border-slate-300" key="1-3">
                        <StockChart
                            default_symbol={"TSLA"}
                            resolution={"1" as ResolutionString}
                            macd_config={[
                                { fast: 4, slow: 9, signal: 9, source: "volume" },
                                { fast: 4, slow: 9, signal: 9 },
                                { fast: 12, slow: 26, signal: 9 },
                            ]}
                            hidden_extra_toolbar={false}
                        />
                    </div>
                    <div className="row-span-3 col-span-4 border-solid border-2 border-slate-300" key="1-4">
                        <StockChart
                            default_symbol={"TSLA"}
                            resolution={"5" as ResolutionString}
                            macd_config={[
                                { fast: 4, slow: 9, signal: 9 },
                                { fast: 12, slow: 26, signal: 9 },
                            ]}
                            hidden_extra_toolbar={true}
                        />
                    </div>
                </div>
                {/* right grids */}
                <div className="col-span-3 grid grid-rows-3">
                    <div key="2-1" className="border-solid border-2 border-slate-300">
                        <StockChart
                            default_symbol={"TSLA"}
                            resolution={"10" as ResolutionString}
                            macd_config={[]}
                            hidden_extra_toolbar={true}
                        />
                    </div>
                    <div key="2-2" className="border-solid border-2 border-slate-300">
                        <StockChart
                            default_symbol={"TSLA"}
                            resolution={"15" as ResolutionString}
                            macd_config={[]}
                            hidden_extra_toolbar={true}
                        />
                    </div>
                    <div key="2-3" className="border-solid border-2 border-slate-300">
                        <StockChart
                            default_symbol={"TSLA"}
                            resolution={"30" as ResolutionString}
                            macd_config={[]}
                            hidden_extra_toolbar={true}
                        />
                    </div>
                </div>
            </div>
        );
    }
}
