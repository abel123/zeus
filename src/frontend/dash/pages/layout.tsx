"use client";

import "@/styles/globals.css";
import { NavBar } from "../app/navbar";
import Head from "next/head";
import { RecoilRoot } from "recoil";
import { RecoilURLSyncJSONNext } from "recoil-sync-next";
import { UDFCompatibleDatafeed } from "@/public/static/datafeeds/udf/src/udf-compatible-datafeed";

export default function RootLayout({ children }: { children: React.ReactNode }) {
    (globalThis as any).datafeed =
        (globalThis as any).datafeed ?? new UDFCompatibleDatafeed("http://127.0.0.1:8080/datafeed/udf", 5 * 1000);
    return (
        <html lang="en">
            <Head>
                <link rel="icon" type="image/png" sizes="32x32" href="/favicon-32x32.png"></link>
                <link rel="icon" type="image/png" sizes="16x16" href="/favicon-16x16.png"></link>
            </Head>
            <body>
                <RecoilRoot>
                    <RecoilURLSyncJSONNext location={{ part: "queryParams" }}>
                        <div id="nav_bar">
                            <NavBar />
                        </div>
                        <div className="h-screen" id="content">
                            {children}
                        </div>
                    </RecoilURLSyncJSONNext>
                </RecoilRoot>
            </body>
        </html>
    );
}
