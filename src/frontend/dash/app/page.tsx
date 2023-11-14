"use client";

import Head from "next/head";
import dynamic from "next/dynamic";
import { useMemo, useState } from "react";
import Script from "next/script";

import {
    ChartingLibraryWidgetOptions,
    ResolutionString,
    LanguageCode,
    version,
} from "@/public/static/charting_library";
import { StockChart } from "./components/widgets/charts/stock";

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

            <div className="">
                <div className="columns-1 border-grey-100	 border-y-2" key="widget">
                    {isScriptReady && (
                        <StockChart
                            default_symbol={"TSLA"}
                            resolution={"1D" as ResolutionString}
                            macd_config={[
                                { fast: 4, slow: 9, signal: 9 },
                                { fast: 12, slow: 26, signal: 9 },
                            ]}
                        />
                    )}
                </div>
            </div>
        </>
    );
}
