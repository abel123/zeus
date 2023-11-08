"use client";

import Head from "next/head";
import dynamic from "next/dynamic";
import { useState } from "react";
import Script from "next/script";

import {
  ChartingLibraryWidgetOptions,
  ResolutionString,
  LanguageCode,
  version,
} from "@/public/static/charting_library";

const getLanguageFromURL = (): LanguageCode | null => {
  if (typeof window != "undefined") {
	  const regex = new RegExp('[\\?&]lang=([^&#]*)');
	  const results = regex.exec(location.search);
	  return results === null ? null : decodeURIComponent(results[1].replace(/\+/g, ' ')) as LanguageCode;
  }

  return "en" as LanguageCode;
};

const defaultWidgetProps: Partial<ChartingLibraryWidgetOptions> = {
  symbol: "AAPL",
  interval: "1D" as ResolutionString,
  library_path: "/static/charting_library/",
  locale: getLanguageFromURL() || 'en',
  charts_storage_url: "https://saveload.tradingview.com",
  charts_storage_api_version: "1.1",
  client_id: "tradingview.com",
  user_id: "public_user_id",
  fullscreen: false,
  autosize: true,
};

const TVChartContainer = dynamic(
  () =>
    import("@/app/components/TVChartContainer").then((mod) => mod.TVChartContainer),
  { ssr: false }
);

export default function Home() {
  const [isScriptReady, setIsScriptReady] = useState(false);
  const symbols = ["AAPL", "SQ"];

  return (
    <>
      <Head>
        <title>TradingView Charting Library</title>
      </Head>
      <Script
        id = "udf-dist"
        key="udf-dist"
        src="/static/datafeeds/udf/dist/bundle.js"
        strategy="lazyOnload"
        onReady={() => {
          setIsScriptReady(true);
        }}
      />
      
      <div className="">
        <ul key="symbol_list"> 
          {["AAPL"].map((sym, i)  =>  {
                console.log(isScriptReady, i);
            if(!isScriptReady){
              return <></>
            }
            
            return <div className="columns-1 border-grey-100	 border-y-2" key={(i*2).toString()}>
              <li key={"li"+(i*2).toString()}> { isScriptReady && <TVChartContainer { ...{...defaultWidgetProps, symbol: sym} } />} </li>
          </div>

          })
          }
        </ul>  
      </div>

    </>
  );
}