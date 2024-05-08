"use client";

import { useMemo, useState } from "react";
import Script from "next/script";
import { LocalGrid } from "@/app/components/dashboard/local_data";
import { DataFeedWrapper } from "@/app/components/widgets/charts/tv_chart/datafeed";

export default function Home() {
    const [isScriptReady, setIsScriptReady] = useState(false);

    (globalThis as any).use_local = true;
    (globalThis as any).datafeed = new DataFeedWrapper("http://192.168.31.180:8080/datafeed/udf", -1);
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

            <>{isScriptReady && <LocalGrid />}</>
        </>
    );
}
