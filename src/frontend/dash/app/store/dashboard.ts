"use client";

import { string } from "@recoiljs/refine";
import { atom } from "recoil";
import { syncEffect } from "recoil-sync";

export const symbolState = atom({
    key: "symbolState", // unique ID (with respect to other atoms/selectors)
    default: "SPY", // default value (aka initial value)
    effects: [syncEffect({ refine: string(), syncDefault: false })],
});

export const optionState = atom({
    key: "optionState", // unique ID (with respect to other atoms/selectors)
    default: "", // default value (aka initial value)
});

export const replayState = atom({
    key: "replayState",
    default: 0,
});
