"use client";
import "@/styles/globals.css";
import type { AppProps } from "next/app";
import { useEffect } from "react";
import RootLayout from "./layout";
import { Metadata } from "next";

export default function App({ Component, pageProps }: AppProps) {
    return (
        <RootLayout>
            <Component {...pageProps} />
        </RootLayout>
    );
}
