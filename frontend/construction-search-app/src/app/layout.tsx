"use client"; // Mark this file as a client component

import "./globals.css";
import { Inter } from "next/font/google";

const inter = Inter({
  subsets: ["latin"],
  display: "swap",
});

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="es">
      <body className={inter.className}>
        <div className="min-h-screen bg-gray-50">
          {/* Header */}
          <header className="bg-white border-b border-gray-200">
            <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
              <div className="flex justify-between items-center h-16">
                <h1 className="text-2xl font-bold text-gray-900">
                  Buscador de Materiales
                </h1>
                <nav className="flex space-x-4">
                  <a href="/" className="text-gray-600 hover:text-gray-900">
                    Inicio
                  </a>
                  <a href="/favoritos" className="text-gray-600 hover:text-gray-900">
                    Favoritos
                  </a>
                </nav>
              </div>
            </div>
          </header>

          {/* Main Content */}
          <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
            {children}
          </main>

          {/* Footer */}
          // ... existing code ...
          <footer className="bg-white dark:bg-gray-900 border-t border-gray-200 dark:border-gray-800">
            <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
              {/* Grid principal */}
              <div className="py-12 grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-8">
                {/* Columna 1: Información de la empresa */}
                <div className="space-y-4">
                  <h3 className="text-lg font-bold text-gray-900 dark:text-white">
                    Buscador de Materiales
                  </h3>
                  <p className="text-sm text-gray-600 dark:text-gray-400 leading-relaxed">
                    La plataforma líder en búsqueda y comparación de materiales de construcción en España.
                  </p>
                  <div className="flex space-x-4 pt-2">
                    <a href="#" className="text-gray-400 hover:text-blue-600 transition-colors">
                      <span className="sr-only">LinkedIn</span>
                      <svg className="h-6 w-6" fill="currentColor" viewBox="0 0 24 24">
                        <path d="M19 0h-14c-2.761 0-5 2.239-5 5v14c0 2.761 2.239 5 5 5h14c2.762 0 5-2.239 5-5v-14c0-2.761-2.238-5-5-5zm-11 19h-3v-11h3v11zm-1.5-12.268c-.966 0-1.75-.79-1.75-1.764s.784-1.764 1.75-1.764 1.75.79 1.75 1.764-.783 1.764-1.75 1.764zm13.5 12.268h-3v-5.604c0-3.368-4-3.113-4 0v5.604h-3v-11h3v1.765c1.396-2.586 7-2.777 7 2.476v6.759z" />
                      </svg>
                    </a>
                    <a href="#" className="text-gray-400 hover:text-blue-500 transition-colors">
                      <span className="sr-only">Twitter</span>
                      <svg className="h-6 w-6" fill="currentColor" viewBox="0 0 24 24">
                        <path d="M23.953 4.57a10 10 0 01-2.825.775 4.958 4.958 0 002.163-2.723c-.951.555-2.005.959-3.127 1.184a4.92 4.92 0 00-8.384 4.482C7.69 8.095 4.067 6.13 1.64 3.162a4.822 4.822 0 00-.666 2.475c0 1.71.87 3.213 2.188 4.096a4.904 4.904 0 01-2.228-.616v.06a4.923 4.923 0 003.946 4.827 4.996 4.996 0 01-2.212.085 4.936 4.936 0 004.604 3.417 9.867 9.867 0 01-6.102 2.105c-.39 0-.779-.023-1.17-.067a13.995 13.995 0 007.557 2.209c9.053 0 13.998-7.496 13.998-13.985 0-.21 0-.42-.015-.63A9.935 9.935 0 0024 4.59z" />
                      </svg>
                    </a>
                  </div>
                </div>

                {/* Columna 2: Enlaces rápidos */}
                <div>
                  <h4 className="text-sm font-semibold text-gray-900 dark:text-white uppercase tracking-wider mb-4">
                    Enlaces rápidos
                  </h4>
                  <ul className="space-y-3">
                    <li>
                      <a href="/busqueda" className="text-gray-600 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 text-sm transition-colors">
                        Búsqueda avanzada
                      </a>
                    </li>
                    <li>
                      <a href="/ofertas" className="text-gray-600 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 text-sm transition-colors">
                        Ofertas especiales
                      </a>
                    </li>
                    <li>
                      <a href="/proveedores" className="text-gray-600 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 text-sm transition-colors">
                        Proveedores
                      </a>
                    </li>
                  </ul>
                </div>

                {/* Columna 3: Legal */}
                <div>
                  <h4 className="text-sm font-semibold text-gray-900 dark:text-white uppercase tracking-wider mb-4">
                    Legal
                  </h4>
                  <ul className="space-y-3">
                    <li>
                      <a href="/privacidad" className="text-gray-600 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 text-sm transition-colors">
                        Política de privacidad
                      </a>
                    </li>
                    <li>
                      <a href="/terminos" className="text-gray-600 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 text-sm transition-colors">
                        Términos y condiciones
                      </a>
                    </li>
                    <li>
                      <a href="/cookies" className="text-gray-600 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 text-sm transition-colors">
                        Política de cookies
                      </a>
                    </li>
                  </ul>
                </div>

                {/* Columna 4: Contacto */}
                <div>
                  <h4 className="text-sm font-semibold text-gray-900 dark:text-white uppercase tracking-wider mb-4">
                    Contacto
                  </h4>
                  <ul className="space-y-3">
                    <li className="text-sm text-gray-600 dark:text-gray-400">
                      <span className="block font-medium">Atención al cliente:</span>
                      900 123 456
                    </li>
                    <li className="text-sm text-gray-600 dark:text-gray-400">
                      <span className="block font-medium">Email:</span>
                      info@buscadormateriales.es
                    </li>
                    <li className="text-sm text-gray-600 dark:text-gray-400">
                      <span className="block font-medium">Horario:</span>
                      Lun-Vie: 9:00-18:00
                    </li>
                  </ul>
                </div>
              </div>

              {/* Línea divisoria */}
              <div className="border-t border-gray-200 dark:border-gray-800">
                <div className="py-6 flex flex-col sm:flex-row justify-between items-center">
                  <p className="text-sm text-gray-500 dark:text-gray-400">
                    © {new Date().getFullYear()} Buscador de Materiales. Todos los derechos reservados.
                  </p>
                  <div className="flex items-center space-x-3 mt-4 sm:mt-0">
                    <img src="/images/ssl-secure.svg" alt="SSL Secure" className="h-8" />
                    <img src="/images/payment-methods.svg" alt="Payment Methods" className="h-8" />
                  </div>
                </div>
              </div>
            </div>
          </footer>
        </div>
      </body>
    </html>
  );
}

