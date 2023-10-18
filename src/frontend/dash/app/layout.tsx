import '@/styles/globals.css'
import type { Metadata } from 'next'
import { Inter } from 'next/font/google'
import { NavBar } from './components/NavBar/navbar';

const inter = Inter({ subsets: ['latin'] })

export const metadata: Metadata = {
  title: 'trading terminal',
  description: 'for trading',
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en">
      <body className={inter.className}>
        <div className="">
          <div id="nav_bar">
            <NavBar/>
          </div>
          {children}
        </div>
        </body>
    </html>
  )
}
