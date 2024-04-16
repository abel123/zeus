"use client";

import { Path, CheckResult, number, string } from "@recoiljs/refine";
import { RecoilRoot, atom, selector, useRecoilState, useRecoilValue } from "recoil";
import { syncEffect } from "recoil-sync";

export const symbolState = atom({
    key: "symbolState", // unique ID (with respect to other atoms/selectors)
    default: "TSLA", // default value (aka initial value)
    effects: [syncEffect({ refine: string() })],
});

export const optionState = atom({
    key: "optionState", // unique ID (with respect to other atoms/selectors)
    default: "", // default value (aka initial value)
});

export const replayState = atom({
    key: "replayState",
    default: 0,
});

export const zenControl = atom({
    key: "enable",
    default: true,
});
