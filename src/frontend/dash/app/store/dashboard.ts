import { RecoilRoot, atom, selector, useRecoilState, useRecoilValue } from "recoil";

export const symbolState = atom({
    key: "symbolState", // unique ID (with respect to other atoms/selectors)
    default: "TSLA", // default value (aka initial value)
});
