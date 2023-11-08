"use client";

import styles from "./index.module.css";
import { useEffect, useRef } from "react";
import { ChartingLibraryWidgetOptions, LanguageCode, ResolutionString, widget,PineJS, CreateTradingViewStyledButtonOptions, Bar, EntityId } from "@/public/static/charting_library";
import DataFeedFactory from "./lib/datafeed";
import { url } from "inspector";
import axios from "axios";
import { debounce } from "lodash"
import React from "react";

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
			autosize: props.autosize,
			symbol_search_request_delay: 1200
			//theme: "Dark",
		};
		const tvWidget = new widget(widgetOptions);
		// @ts-ignore
		window.tvWidget = tvWidget;
		
		tvWidget.onChartReady(() => {
			tvWidget.subscribe('chart_load_requested', (event) => {
				console.log('chart_load_requested', event);
			  });

			tvWidget.headerReady().then(()=>{
				const button = tvWidget.createButton();
				button.innerHTML = "Draw Zen";
				button.addEventListener("click", () =>{

				}
				);
			});

			let chart = tvWidget.activeChart();
			chart.createStudy("MACD").then((macd_indicator_id) => {
				function draw_zen(){
					let range = chart.getVisibleRange();
					let symbol = chart.symbolExt();
					let resolution = chart.resolution();
					if(range.from >= range.to){
						return
					}

					axios.get("http://127.0.0.1:8000/zen/elements", {"params":{ 
						"from": range.from,
						"to": range.to,
						"symbol": symbol?.symbol,
						"resolution": resolution,
					}}).then((response) => {
						chart.removeAllShapes();
						chart.getAllShapes().map((info) => {
							// TODO, compare shape, generate diff to reduce duplicate shape drawing
						})
						console.log("response", response.data);
						response.data['bi']['finished'].map((bi: any) => {
							chart.createMultipointShape([{price: bi.start, time: bi.start_ts}, {price:bi.end, time: bi.end_ts}], {
								shape: 'trend_line',
								disableSelection: true,
								text: "test"
							});
						})
						response.data['bi']['unfinished'].map((bi: any) => {
							chart.createMultipointShape([{price: bi.start, time: bi.start_ts}, {price:bi.end, time: bi.end_ts}], {
								shape: 'trend_line',
								disableSelection: true,
								text: "test",
								overrides:{
									//linestyle: 1,
									linewidth: 1,
									linecolor: '#ff7373',
								}
							});
						})

						response.data['beichi'].map((bc: any) => {
							chart.createMultipointShape([{price: bc.macd_a_val, time: bc.macd_a_dt}, {price:bc.macd_b_val, time: bc.macd_b_dt}], {
								shape: 'arrow',
								//disableSelection: true,
								text: "test",
								overrides:{
									//linestyle: 1,
									linewidth: 2,
									linecolor: bc.direction == "down"?'#ff1493':'#00ce09',
								},
								ownerStudyId: macd_indicator_id as EntityId,
								zOrder: "top"
							});
						})
					}).catch(function (error) {
						console.log(error);
					  })
					  .finally(function () {
						// always executed
					  }); 
				}
				const debounced_draw_zen = 
					debounce(async () => {
					  draw_zen();
					}, 100);
				chart.onDataLoaded().subscribe(null, ()=>{
					console.log("on data loaded");
					debounced_draw_zen();
				}, true)
				tvWidget.subscribe("onTick", (tick: Bar)=> {
					console.log("on tick");
					debounced_draw_zen();
				});
				chart.onVisibleRangeChanged().subscribe(null, function (visible_range) {
					console.log("range change");
                    debounced_draw_zen();
                });

			});
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