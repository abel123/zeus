import "@/styles/globals.css";
import type { AppProps } from "next/app";
import { useEffect } from 'react';

export default function App({ Component, pageProps }: AppProps) {
    useEffect(() => {
        // @ts-ignore

        import('preline')
      }, [])

    return <Component {...pageProps} />;
}