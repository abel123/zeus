export interface MacdConfig {
    fast: number;
    slow: number;
    signal: number;
    source?: string;
}

export interface Zen {
    bi: Bi;
    beichi: Beichi[][];
    bar_beichi: number[][];
}

export interface Bi {
    finished: BiInfo[];
    unfinished: BiInfo[];
}

export interface BiInfo {
    start_ts: number;
    end_ts: number;
    start: number;
    end: number;
    direction: string;
}

export interface Beichi {
    macd_a_dt: number;
    macd_a_val: number;
    macd_b_dt: number;
    macd_b_val: number;
    direction: string;
    start: Start;
    end: End;
    high: number;
    low: number;
}

export interface Start {
    left_dt: number;
    right_dt: number;
}

export interface End {
    left_dt: number;
    right_dt: number;
}
