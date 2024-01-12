"use client";

import { StockChart } from "@/app/components/widgets/charts/stock";
import { ResolutionString } from "@/public/static/charting_library/charting_library";
import Script from "next/script";
import { useState } from "react";

export default function Home() {
    const [isScriptReady, setIsScriptReady] = useState(false);

    return (
        <>
            <Script
                id="udf-dist"
                key="udf-dist"
                src="/static/datafeeds/udf/dist/bundle.js"
                strategy="lazyOnload"
                onReady={() => {
                    setIsScriptReady(true);
                }}
            />

            <div className="grid grid-cols-6 bg-white h-full">
                <div className="col-span-4 border-solid border-2 border-slate-300" key="1-1">
                    {isScriptReady && (
                        <StockChart
                            resolution={"1D" as ResolutionString}
                            macd_config={[
                                { fast: 4, slow: 9, signal: 9, source: "volume" },
                                { fast: 4, slow: 9, signal: 9 },
                                { fast: 12, slow: 26, signal: 9 },
                            ]}
                            hidden_extra_toolbar={false}
                            standalone={false}
                        />
                    )}
                </div>
                <div className="col-span-2 border-solid border-2 border-slate-300" key="1-2">
                    {isScriptReady && (
                        <StockChart
                            resolution={"60" as ResolutionString}
                            macd_config={[
                                { fast: 4, slow: 9, signal: 9, source: "volume" },
                                { fast: 4, slow: 9, signal: 9 },
                                { fast: 12, slow: 26, signal: 9 },
                            ]}
                            hidden_extra_toolbar={false}
                            standalone={false}
                        />
                    )}
                </div>
            </div>
        </>
    );
}
