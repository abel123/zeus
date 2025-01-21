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
    bc_type: string[];
    macd_a_dt: number;
    macd_a_val: number;
    macd_b_dt: number;
    macd_b_val: number;
    direction: string;
    type: string;
    zs1: ZS;
    zs2: ZS;
    dt: number;
    price: number;
    bi_count: number;
}

export interface ZS {
    left: number;
    right: number;
    high: number;
    low: number;
    bi_count: number;
}

export interface Start {
    left_dt: number;
    right_dt: number;
}

export interface End {
    left_dt: number;
    right_dt: number;
}
