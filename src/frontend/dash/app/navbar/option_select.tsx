import { useRecoilState } from "recoil";
import { optionState, symbolState } from "../store/dashboard";
import { useEffect, useState } from "react";
import axios from "axios";

export const OptionSelect = () => {
    const [symbol, setSymbol] = useRecoilState(symbolState);
    const [option, setOption] = useRecoilState(optionState);
    const [select_options, setSelectOption] = useState([]);

    useEffect(() => {
        axios
            .get(`http://127.0.0.1:8080/datafeed/udf/search?limit=30&query=${symbol}%20&type=&exchange=america`)
            .then(function (res) {
                console.log("option response", res);
                setSelectOption(res.data);
            })
            .catch(function (err) {
                console.log(err);
            });
    }, [symbol]);
    return (
        <select
            className="select select-primary w-full max-w-xs"
            onChange={(event) => {
                console.log("stock option selected", event.target.value);
                setOption(event.target.value);
            }}
        >
            <option disabled selected>
                Stock Option
            </option>
        </select>
    );
};
