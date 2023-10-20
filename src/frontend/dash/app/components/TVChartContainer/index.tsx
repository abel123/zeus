"use client";

import styles from "./index.module.css";
import { useEffect, useRef } from "react";
import { ChartingLibraryWidgetOptions, LanguageCode, ResolutionString, widget,PineJS } from "@/public/static/charting_library";
import DataFeedFactory from "./lib/datafeed";
import { url } from "inspector";

export const TVChartContainer = (props: Partial<ChartingLibraryWidgetOptions>) => {
	const chartContainerRef =
		useRef<HTMLDivElement>() as React.MutableRefObject<HTMLInputElement>;

	useEffect(() => {
		const widgetOptions: ChartingLibraryWidgetOptions = {
			symbol: props.symbol,
			// BEWARE: no trailing slash is expected in feed URL
			datafeed: (globalThis as any).datafeed ?? DataFeedFactory("http://127.0.0.1:8000"),
			interval: props.interval as ResolutionString,
			container: chartContainerRef.current,
			library_path: props.library_path,
			locale: props.locale as LanguageCode,
			disabled_features: ["use_localstorage_for_settings"],
			enabled_features: ["study_templates"],
			charts_storage_url: props.charts_storage_url,
			charts_storage_api_version: props.charts_storage_api_version,
			client_id: props.client_id,
			user_id: props.user_id,
			fullscreen: props.fullscreen,
			autosize: props.autosize
		};
		const tvWidget = new widget(widgetOptions);

		tvWidget.onChartReady(() => {
		
		});

		return () => {
			tvWidget.remove();
		};
	}, [props]);

	return (
		<>
			<div ref={chartContainerRef} className={styles.TVChartContainer} />
		</>
	);
};