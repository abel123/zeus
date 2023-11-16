"use client";

import { useMemo, useState } from "react";
import Script from "next/script";

import { MainGrid } from "../app/components/dashboard";

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

            <>{isScriptReady && <MainGrid />}</>
        </>
    );
}
