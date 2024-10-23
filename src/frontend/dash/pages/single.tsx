"use client";

import { StockOptionTable } from "@/app/components/stock/stock_option_table";
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

            <div className="grid grid-cols-7 bg-white h-full">
                <div className="col-span-7 border-solid border-2 border-slate-300" key="3-2">
                    {isScriptReady && (
                        <StockChart
                            resolution={"60" as ResolutionString}
                            macd_config={[
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
