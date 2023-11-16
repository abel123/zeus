"use client";
import "@/styles/globals.css";
import type { Metadata } from "next";
import { Inter } from "next/font/google";
import { NavBar } from "../app/navbar";

export default function RootLayout({ children }: { children: React.ReactNode }) {
    return (
        <html lang="en">
            <body className="h-screen">
                <>
                    <div id="nav_bar">
                        <NavBar />
                    </div>
                </>
                {children}
            </body>
        </html>
    );
}
