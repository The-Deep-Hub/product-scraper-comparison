"use client"; // Mark this file as a client component

import "./globals.css";
import { Inter } from "next/font/google";
import { useState } from 'react';

const inter = Inter({
  subsets: ["latin"],
  display: "swap",
});

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  const [isFooterOpen, setIsFooterOpen] = useState(false);

  return (
    <html lang="es">
      <body className="flex flex-col min-h-screen">
        <main className="flex-grow">
          {children}
        </main>
        
        <div className="relative">
          {/* Botón flotante circular */}
          <button
            onClick={() => setIsFooterOpen(!isFooterOpen)}
            className="absolute -top-16 right-8 w-12 h-12 bg-blue-600 hover:bg-blue-700 text-white rounded-full flex items-center justify-center shadow-lg hover:shadow-xl transition-all duration-300 group"
            aria-label="Toggle footer"
          >
            <svg
              className={`w-6 h-6 transition-transform duration-300 ${
                isFooterOpen ? 'rotate-180' : ''
              }`}
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 15l7-7 7 7" />
            </svg>
          </button>

          {/* Footer rediseñado */}
          <footer 
            className={`bg-gradient-to-br from-gray-900 to-gray-800 transition-all duration-300 ease-in-out ${
              isFooterOpen ? 'max-h-[800px] opacity-100' : 'max-h-0 opacity-0'
            } overflow-hidden`}
          >
            <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-16">
              {/* Grid principal con nuevo diseño */}
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-12 gap-8 lg:gap-12">
                {/* Columna grande de información */}
                <div className="lg:col-span-5 space-y-6">
                  <h3 className="text-2xl font-bold text-white">
                    Buscador de Materiales
                  </h3>
                  <p className="text-gray-300 leading-relaxed max-w-md">
                    Transformando la manera en que los profesionales encuentran y comparan materiales de construcción en España.
                  </p>
                  <div className="flex items-center space-x-6 pt-4">
                    {['facebook', 'twitter', 'linkedin', 'instagram'].map((social) => (
                      <a
                        key={social}
                        href={`#${social}`}
                        className="text-gray-400 hover:text-blue-400 transition-colors duration-200"
                      >
                        <span className="sr-only">{social}</span>
                        <div className="w-8 h-8 rounded-full bg-gray-800 flex items-center justify-center hover:bg-gray-700 transition-colors">
                          <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                            {/* Aquí irían los paths específicos de cada red social */}
                          </svg>
                        </div>
                      </a>
                    ))}
                  </div>
                </div>

                {/* Enlaces y contacto en diseño minimalista */}
                <div className="lg:col-span-7 grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-8">
                  {/* Enlaces rápidos */}
                  <div>
                    <h4 className="text-blue-400 font-medium uppercase tracking-wider text-sm mb-4">
                      Explorar
                    </h4>
                    <ul className="space-y-3">
                      {['Búsqueda avanzada', 'Ofertas', 'Proveedores', 'Blog'].map((item) => (
                        <li key={item}>
                          <a href="#" className="text-gray-300 hover:text-white text-sm transition-colors duration-200">
                            {item}
                          </a>
                        </li>
                      ))}
                    </ul>
                  </div>

                  {/* Legal */}
                  <div>
                    <h4 className="text-blue-400 font-medium uppercase tracking-wider text-sm mb-4">
                      Legal
                    </h4>
                    <ul className="space-y-3">
                      {['Privacidad', 'Términos', 'Cookies'].map((item) => (
                        <li key={item}>
                          <a href="#" className="text-gray-300 hover:text-white text-sm transition-colors duration-200">
                            {item}
                          </a>
                        </li>
                      ))}
                    </ul>
                  </div>

                  {/* Contacto con diseño minimalista */}
                  <div>
                    <h4 className="text-blue-400 font-medium uppercase tracking-wider text-sm mb-4">
                      Contacto
                    </h4>
                    <ul className="space-y-3">
                      <li>
                        <a href="tel:900123456" className="text-gray-300 hover:text-white text-sm transition-colors duration-200">
                          900 123 456
                        </a>
                      </li>
                      <li>
                        <a href="mailto:info@buscadormateriales.es" className="text-gray-300 hover:text-white text-sm transition-colors duration-200">
                          info@buscadormateriales.es
                        </a>
                      </li>
                    </ul>
                  </div>
                </div>
              </div>

              {/* Línea divisoria con nuevo estilo */}
              <div className="mt-12 pt-8 border-t border-gray-700">
                <div className="flex flex-col md:flex-row justify-between items-center">
                  <p className="text-sm text-gray-400">
                    © {new Date().getFullYear()} Buscador de Materiales. Todos los derechos reservados.
                  </p>
                  {/* Badges con nuevo diseño */}
                  <div className="flex items-center space-x-4 mt-4 md:mt-0">
                    <span className="px-3 py-1 text-xs text-blue-400 bg-gray-800 rounded-full">
                      SSL Secure
                    </span>
                    <span className="px-3 py-1 text-xs text-blue-400 bg-gray-800 rounded-full">
                      Verified Partner
                    </span>
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
