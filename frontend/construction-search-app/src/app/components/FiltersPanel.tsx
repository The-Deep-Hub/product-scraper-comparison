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
    <aside className="sticky top-4 w-full md:w-1/4 bg-white dark:bg-gray-800 p-6 rounded-xl shadow-lg border border-gray-200 dark:border-gray-700">
      <h2 className="text-xl font-bold mb-6 text-gray-800 dark:text-white">Filtros</h2>

      {/* Price Range Filter */}
      <div className="mb-8">
        <label className="block font-semibold mb-3 text-gray-700 dark:text-gray-200">Rango de precio:</label>

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
        <div className="flex justify-between mt-3 text-sm font-medium text-gray-600 dark:text-gray-300">
          <span>Mín: {formatPrice(priceRange[0])}</span>
          <span>Máx: {formatPrice(priceRange[1])}</span>
        </div>
      </div>

      {/* Sort Order */}
      <div className="mb-8">
        <button
          onClick={() => handleSortChange(sortOrder === 'desc' ? 'asc' : 'desc')}
          className="w-full px-4 py-3 text-sm font-medium rounded-lg bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-200 hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
        >
          Ordenar: {sortOrder === 'desc' ? 'Mayor a menor' : 'Menor a mayor'}
        </button>
      </div>

      {/* Provider Filter */}
      <div className="mb-8">
        <label className="block font-semibold mb-4 text-gray-700 dark:text-gray-200">Proveedores:</label>
        {PROVIDER_OPTIONS.map((provider) => (
          <div key={provider.key} className="flex items-center mb-3 p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors">
            <input
              type="checkbox"
              id={provider.key}
              checked={selectedProviders.includes(provider.key)}
              onChange={() => handleProviderChange(provider.key)}
              className="w-4 h-4 mr-3 accent-green-500 cursor-pointer"
            />
            <label htmlFor={provider.key} className="text-sm cursor-pointer select-none text-gray-700 dark:text-gray-200">
              {provider.label}
            </label>
          </div>
        ))}
      </div>

      {/* Offers Filter */}
      <div className="mb-6">
        <button
          onClick={toggleOffers}
          className="w-full px-4 py-3 text-sm font-medium rounded-lg bg-green-100 dark:bg-green-900 text-green-700 dark:text-green-200 hover:bg-green-200 dark:hover:bg-green-800 transition-colors"
        >
          {showOnlyOffers ? 'Ver todos los productos' : 'Solo ofertas'}
        </button>
      </div>

      {/* Message when no products are available */}
      {maxPrice === 0 && (
        <p className="text-sm text-gray-500 dark:text-gray-400 mt-4 text-center italic">
          No hay productos disponibles para filtrar.
        </p>
      )}
    </aside>
  );
};

export default FilterPanel;
