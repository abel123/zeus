"use client";

import { Path, CheckResult, number, string } from "@recoiljs/refine";
import { RecoilRoot, atom, selector, useRecoilState, useRecoilValue } from "recoil";
import { syncEffect } from "recoil-sync";
export const symbolState = atom({
    key: "symbolState", // unique ID (with respect to other atoms/selectors)
    default: "TSLA", // default value (aka initial value)
    effects: [syncEffect({ refine: string() })],
});
