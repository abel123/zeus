import "@/styles/globals.css";
import { NavBar } from "../app/navbar";
import Head from "next/head";
import { RecoilRoot } from "recoil";

export default function RootLayout({ children }: { children: React.ReactNode }) {
    return (
        <html lang="en">
            <Head>
                <link rel="icon" type="image/png" sizes="32x32" href="/favicon-32x32.png"></link>
                <link rel="icon" type="image/png" sizes="16x16" href="/favicon-16x16.png"></link>
            </Head>
            <body className="h-screen">
                <RecoilRoot>
                    <>
                        <div id="nav_bar">
                            <NavBar />
                        </div>
                    </>
                    {children}
                </RecoilRoot>
            </body>
        </html>
    );
}
