import React, { useState, useRef, useEffect } from 'react';

type SearchConfigProps = {
  selectedStores: string[];
  onStoreChange: (store: string) => void;
  productsPerStore: number;
  onProductsPerStoreChange: (value: number) => void;
};

const STORE_OPTIONS = [
  { key: "leroy", label: "Leroy Merlin" },
  { key: "bauhaus", label: "Bauhaus" },
  { key: "bricodepot", label: "Bricodepot" },
];

const SearchConfig: React.FC<SearchConfigProps> = ({
  selectedStores,
  onStoreChange,
  productsPerStore,
  onProductsPerStoreChange,
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  return (
    <div className="relative inline-block" ref={dropdownRef}>
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="p-2 hover:bg-gray-100 rounded-lg text-gray-700 focus:outline-none"
        title="Configurar búsqueda"
      >
        <svg
          className={`w-5 h-5 transition-transform ${isOpen ? 'rotate-180' : ''}`}
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
        </svg>
      </button>

      {isOpen && (
        <div className="absolute right-0 mt-2 bg-white rounded-lg shadow-lg border border-gray-200 p-4 z-50 min-w-[250px]">
          <div className="space-y-4">
            <div>
              <h3 className="text-sm font-semibold mb-2">Seleccionar Tiendas:</h3>
              {STORE_OPTIONS.map((store) => (
                <div key={store.key} className="flex items-center space-x-2 mb-2">
                  <input
                    type="checkbox"
                    id={`search-${store.key}`}
                    checked={selectedStores.includes(store.key)}
                    onChange={() => onStoreChange(store.key)}
                    className="w-4 h-4 accent-blue-500"
                  />
                  <label htmlFor={`search-${store.key}`} className="text-sm">
                    {store.label}
                  </label>
                </div>
              ))}
            </div>

            <div>
              <h3 className="text-sm font-semibold mb-2">Productos por tienda:</h3>
              <input
                type="number"
                min="1"
                max="20"
                value={productsPerStore}
                onChange={(e) => onProductsPerStoreChange(Math.max(1, Math.min(20, parseInt(e.target.value) || 1)))}
                className="w-full px-2 py-1 border rounded text-sm"
              />
              <p className="text-xs text-gray-500 mt-1">Máximo 20 productos por tienda</p>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default SearchConfig; 