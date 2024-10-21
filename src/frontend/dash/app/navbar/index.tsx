"use client";

import Link from "next/link";
import { FullscreenButton } from "../components/widgets/button/fullscreen";
import { useRecoilState } from "recoil";
import { symbolState } from "../store/dashboard";
import { OptionSelect } from "./option_select";

export const NavBar = () => {
    const [symbol, setSymbol] = useRecoilState(symbolState);

    return (
        <div className="navbar bg-white h-8 px-4 py-2 min-h-0">
            <div className="navbar-start">
                <div className="dropdown">
                    <button tabIndex={0} className="btn-sm btn-ghost btn-circle">
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            className="h-5 w-5"
                            fill="none"
                            viewBox="0 0 24 24"
                            stroke="currentColor"
                        >
                            <path
                                strokeLinecap="round"
                                strokeLinejoin="round"
                                strokeWidth="2"
                                d="M4 6h16M4 12h16M4 18h7"
                            />
                        </svg>
                    </button>
                    <ul
                        tabIndex={0}
                        className="menu menu-sm dropdown-content mt-5 z-[1] shadow bg-slate-100 rounded-box w-100"
                    >
                        <li className="p-2 border-2 border-sky-300 rounded">
                            <Link href="/">
                                <p className="text-xl">Home</p>
                            </Link>
                        </li>
                        <li className="p-2 border-2 border-sky-300 rounded">
                            <Link href="/research">
                                <p className="text-xl">Research</p>
                            </Link>
                        </li>
                        <li>
                            <details>
                                <summary>Parent</summary>
                                <ul>
                                    <li>
                                        <a>level 2 item 1</a>
                                    </li>
                                    <li>
                                        <a>level 2 item 2</a>
                                    </li>
                                    <li>
                                        <details open>
                                            <summary>Parent</summary>
                                            <ul>
                                                <li>
                                                    <a>level 3 item 1</a>
                                                </li>
                                                <li>
                                                    <a>level 3 item 2</a>
                                                </li>
                                            </ul>
                                        </details>
                                    </li>
                                </ul>
                            </details>
                        </li>
                        <li>
                            <a>Item 3</a>
                        </li>
                    </ul>
                </div>
                <input
                    type="text"
                    placeholder="Symbol"
                    className="input input-bordered input-primary w-full max-w-xs"
                    onChange={(event) => {
                        if (event.target.value != "") setSymbol(event.target.value);
                    }}
                />
            </div>
            <div className="navbar-center">
                <a className="normal-case text-xl">Terminal</a>
            </div>
            <div className="navbar-end">
                <input
                    type="checkbox"
                    id="scales"
                    onChange={(ev) => {
                        (globalThis as any).zenUpdate = ev.target.checked;
                        console.log("zen update", (globalThis as any).zenUpdate);
                    }}
                    name="scales"
                    defaultChecked={true}
                />
                <label htmlFor="scales">Update Zen</label>
                <FullscreenButton></FullscreenButton>
                <button className="btn-sm btn-ghost btn-circle">
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        className="h-5 w-5"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke="currentColor"
                    >
                        <path
                            strokeLinecap="round"
                            strokeLinejoin="round"
                            strokeWidth="2"
                            d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                        />
                    </svg>
                </button>
                <button className="btn-sm btn-ghost btn-circle">
                    <div className="indicator">
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            className="h-5 w-5"
                            fill="none"
                            viewBox="0 0 24 24"
                            stroke="currentColor"
                        >
                            <path
                                strokeLinecap="round"
                                strokeLinejoin="round"
                                strokeWidth="2"
                                d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9"
                            />
                        </svg>
                        <span className="badge badge-xs badge-primary indicator-item"></span>
                    </div>
                </button>
            </div>
        </div>
    );
};
