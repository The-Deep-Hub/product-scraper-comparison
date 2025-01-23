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
    <aside className="responsive top-4 w-full md:w-1/4 bg-gray-100 p-4 rounded-md shadow-md">
      <h2 className="text-lg font-bold mb-4">Filtros</h2>

      {/* Price Range Filter */}
      <div className="mb-6">
        <label className="block font-semibold mb-2">Rango de precio:</label>

        {/* Price range slider component */}
        <Range
          step={1}
          min={0}
          max={maxPrice}
          values={priceRange}
          onChange={handlePriceRangeChange}
          renderTrack={({ props, children }) => {
            return (
              <div {...props} className="w-full h-2 bg-gray-300 rounded-md relative">
                <div
                  style={{
                    position: "absolute",
                    left: `${((priceRange[0] - 0) / maxPrice) * 100}%`,
                    right: `${100 - ((priceRange[1] - 0) / maxPrice) * 100}%`,
                    backgroundColor: "blue",
                    height: "100%",
                    borderRadius: "4px",
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
                className={`w-4 h-4 ${
                  isDragged ? "bg-blue-700" : "bg-blue-500"
                } rounded-full shadow-md`}
              />
            );
          }}
        />

        {/* Display selected price range */}
        <div className="flex justify-between text-sm mt-2">
          <span>Mín: {formatPrice(priceRange[0])}</span>
          <span>Máx: {formatPrice(priceRange[1])}</span>
        </div>
      </div>

      {/* Sort Order */}
      <div className="mb-6">
        <button
          onClick={() => handleSortChange(sortOrder === 'asc' ? 'desc' : 'asc')}
          className="w-full px-4 py-2 text-sm rounded-md bg-gray-200 hover:bg-gray-300"
        >
          Ordenar: {sortOrder ? (sortOrder === 'desc' ? 'Mayor a menor' : 'Menor a mayor') : 'Menor a mayor'}
        </button>
      </div>

      {/* Provider Filter */}
      <div className="mb-6">
        <label className="block font-semibold mb-3 text-lg">Proveedores:</label>
        {PROVIDER_OPTIONS.map((provider) => (
          <div key={provider.key} className="flex items-center mb-3 hover:bg-gray-200 p-2 rounded">
            <input
              type="checkbox"
              id={provider.key}
              checked={selectedProviders.includes(provider.key)}
              onChange={() => handleProviderChange(provider.key)}
              className="w-4 h-4 mr-3 accent-blue-500"
            />
            <label htmlFor={provider.key} className="text-sm cursor-pointer select-none">
              {provider.label}
            </label>
          </div>
        ))}
      </div>

      {/* Offers Filter */}
      <div className="mb-6">
        <button
          onClick={toggleOffers}
          className={`w-full px-4 py-2 text-sm rounded-md ${
            showOnlyOffers ? 'bg-blue-500 text-white' : 'bg-gray-200 hover:bg-gray-300'
          }`}
        >
          {showOnlyOffers ? 'Ver todos los productos' : 'Solo ofertas'}
        </button>
      </div>

      {/* Message when no products are available */}
      {maxPrice === 0 && (
        <p className="text-sm text-gray-600 mt-4">No hay productos disponibles para filtrar.</p>
      )}
    </aside>
  );
};

export default FilterPanel;
