import { Range } from "react-range";

// Props type definition for FilterPanel component
type FilterPanelProps = {
  maxPrice: number;  // Maximum price of products available in state
  priceRange: [number, number];  // Selected price range
  formatPrice: (price: number) => string;  // Function to format prices for display
  handlePriceRangeChange: (values: number[]) => void;  // Handler to update price range
  selectedProviders: string[]; // Selected providers
  handleProviderChange: (provider: string) => void; // Handler for provider checkbox change
  showOnlyOffers: boolean; // Whether to show only products with offers
  toggleOffers: () => void; // Handler to toggle offers filter
  sortOrder: 'asc' | 'desc' | null; // Current sort order
  handleSortChange: (order: 'asc' | 'desc') => void; // Handler for sort order change
};

const FilterPanel: React.FC<FilterPanelProps> = ({
  maxPrice,
  priceRange,
  formatPrice,
  handlePriceRangeChange,
  selectedProviders,
  handleProviderChange,
  showOnlyOffers,
  toggleOffers,
  sortOrder,
  handleSortChange
}) => {
  // Provider options with real names
  const PROVIDER_OPTIONS = [
    { key: "leroy", label: "Leroy Merlin" },
    { key: "bauhaus", label: "Bauhaus" },
    { key: "bricodepot", label: "Bricodepot" },
  ];

  return (
    <aside className="sticky top-4 w-full md:w-1/4 bg-white dark:bg-gray-800 p-8 rounded-2xl shadow-xl border border-gray-200 dark:border-gray-700">
      <h2 className="text-2xl font-bold mb-8 text-gray-900 dark:text-white tracking-tight">Filtros</h2>

      {/* Price Range Filter */}
      <div className="mb-10">
        <label className="block text-base font-semibold mb-4 text-gray-800 dark:text-gray-100">
          Rango de precio:
        </label>

        {/* Price range slider component */}
        <Range
          step={1}
          min={0}
          max={maxPrice}
          values={priceRange}
          onChange={handlePriceRangeChange}
          renderTrack={({ props, children }) => {
            const { key, ...restProps } = props;
            return (
              <div key={key} {...restProps} className="w-full h-3 bg-gray-200 dark:bg-gray-600 rounded-lg relative">
                <div
                  style={{
                    position: "absolute",
                    left: `${((priceRange[0] - 0) / maxPrice) * 100}%`,
                    right: `${100 - ((priceRange[1] - 0) / maxPrice) * 100}%`,
                    backgroundColor: "#4CAF50",
                    height: "100%",
                    borderRadius: "0.5rem",
                  }}
                />
                {children}
              </div>
            );
          }}
          renderThumb={({ props, isDragged }) => {
            const { key, ...restProps } = props;
            return (
              <div
                key={key}
                {...restProps}
                className={`w-5 h-5 ${
                  isDragged ? "bg-green-700" : "bg-green-500"
                } rounded-full shadow-lg border-2 border-white dark:border-gray-800`}
              />
            );
          }}
        />

        {/* Display selected price range */}
        <div className="flex justify-between mt-4 text-sm font-medium text-gray-700 dark:text-gray-200">
          <span>Mín: {formatPrice(priceRange[0])}</span>
          <span>Máx: {formatPrice(priceRange[1])}</span>
        </div>
      </div>

      {/* Sort Order */}
      <div className="mb-10">
        <button
          onClick={() => handleSortChange(sortOrder === 'desc' ? 'asc' : 'desc')}
          className="w-full px-5 py-3.5 text-sm font-semibold rounded-xl bg-gray-50 dark:bg-gray-700 text-gray-800 dark:text-gray-100 hover:bg-gray-100 dark:hover:bg-gray-600 transition-all duration-200"
        >
          Ordenar: {sortOrder === 'desc' ? 'Mayor a menor' : 'Menor a mayor'}
        </button>
      </div>

      {/* Provider Filter */}
      <div className="mb-10">
        <label className="block text-base font-semibold mb-5 text-gray-800 dark:text-gray-100">
          Proveedores:
        </label>
        {PROVIDER_OPTIONS.map((provider) => (
          <div key={provider.key} className="flex items-center mb-4 p-3 rounded-xl hover:bg-gray-50 dark:hover:bg-gray-700 transition-all duration-200">
            <input
              type="checkbox"
              id={provider.key}
              checked={selectedProviders.includes(provider.key)}
              onChange={() => handleProviderChange(provider.key)}
              className="w-5 h-5 mr-4 accent-blue-600 cursor-pointer rounded"
            />
            <label htmlFor={provider.key} className="text-base cursor-pointer select-none text-gray-700 dark:text-gray-200">
              {provider.label}
            </label>
          </div>
        ))}
      </div>

      {/* Offers Filter */}
      <div className="mb-8">
        <button
          onClick={toggleOffers}
          className="w-full px-5 py-3.5 text-sm font-semibold rounded-xl bg-blue-50 dark:bg-blue-900/30 text-blue-700 dark:text-blue-200 hover:bg-blue-100 dark:hover:bg-blue-900/50 transition-all duration-200"
        >
          {showOnlyOffers ? 'Ver todos los productos' : 'Solo ofertas'}
        </button>
      </div>

      {/* Message when no products are available */}
      {maxPrice === 0 && (
        <p className="text-sm text-gray-500 dark:text-gray-400 mt-6 text-center italic">
          No hay productos disponibles para filtrar.
        </p>
      )}
    </aside>
  );
};

export default FilterPanel;
