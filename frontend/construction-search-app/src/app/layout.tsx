"use client"; // Mark this file as a client component

import { Geist, Geist_Mono } from "next/font/google";
import "./globals.css";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body
        className={`${geistSans.variable} ${geistMono.variable} antialiased bg-background text-foreground flex flex-col min-h-screen`}
      >
        {/* Header */}
        <header className="sticky top-0 z-10 bg-gray-200 p-4 shadow-md">
          <h1 className="text-2xl font-bold text-center">
            Construction Search App
          </h1>
        </header>

        {/* Main Content */}
        <main className="flex-1 w-full max-w-[1920px] mx-auto px-4 sm:px-8 md:px-12 lg:px-20 xl:px-32 2xl:px-48">
          {children}
        </main>

        {/* Footer */}
        <footer className="sticky p-4 bg-gray-200 text-center">
          <p>© 2024 Construction Search App</p>
          <ul className="flex justify-center gap-4 mt-2 text-sm">
            <li>
              <a href="/privacy" className="text-primary hover:underline">
                Privacy Policy
              </a>
            </li>
            <li>
              <a href="/terms" className="text-primary hover:underline">
                Terms of Service
              </a>
            </li>
            <li>
              <a href="/contact" className="text-primary hover:underline">
                Contact
              </a>
            </li>
          </ul>
        </footer>
      </body>
    </html>
  );
}

