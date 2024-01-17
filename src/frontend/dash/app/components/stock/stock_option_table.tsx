import { optionState, symbolState } from "@/app/store/dashboard";
import axios from "axios";
import { useEffect, useState } from "react";
import { useRecoilState } from "recoil";

export const StockOptionTable = () => {
    const [symbol, setSymbol] = useRecoilState(symbolState);
    const [option, setOption] = useRecoilState(optionState);

    const [data, updater] = useState([]);
    useEffect(() => {
        let interval = setInterval(() => {
            const res = axios
                .post(`http://127.0.0.1:8000/ma/option_price`, {
                    intervals: [3, 5, 15, 60],
                    ma: [15, 30, 60, 120, 200],
                    symbol: symbol,
                    option: option,
                })
                .then(function (response) {
                    console.log(response);
                    updater(response.data);
                })
                .catch(function (error) {
                    console.log(error);
                });
        }, 3000);
        return () => {
            clearInterval(interval);
        };
    }, [symbol, option]);

    return (
        <div className="flex flex-col text-center border-0 border-slate-100 bg-white h-full overflow-scroll">
            <div className="flex flex-row">
                <div className="basis-1/5 row-span-1 border-2">interval</div>
                <div className="basis-1/5 row-span-1 border-2">ma</div>
                <div className="basis-1/5 row-span-1 border-2">delta</div>
                <div className="basis-1/5 row-span-1 border-2">Price</div>
                <div className="basis-1/5 row-span-1 border-2">Option Price</div>
            </div>

            {data.map((row: any, index) => {
                return (
                    <div key={index} className="flex flex-row">
                        <div className="basis-1/5 row-span-1 border-2">{row.interval}</div>
                        <div className="basis-1/5 row-span-1 border-2">{row.ma}</div>
                        <div className="basis-1/5 row-span-1 border-2">{row.delta.toFixed(2)}</div>
                        <div className="basis-1/5 row-span-1 border-2">{row.price.toFixed(2)}</div>
                        <div className="basis-1/5 row-span-1 border-2">{row.option_price.toFixed(2)}</div>
                    </div>
                );
            })}
        </div>
    );
};
