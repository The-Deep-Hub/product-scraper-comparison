import { Range } from "react-range";

// Props type definition for FilterPanel component
type FilterPanelProps = {
  maxPrice: number;  // Maximum price of products available in state
  priceRange: [number, number];  // Selected price range
  formatPrice: (price: number) => string;  // Function to format prices for display
  handlePriceRangeChange: (values: number[]) => void;  // Handler to update price range
  selectedProviders: string[]; // Selected providers
  handleProviderChange: (provider: string) => void; // Handler for provider checkbox change
};

const FilterPanel: React.FC<FilterPanelProps> = ({
  maxPrice,
  priceRange,
  formatPrice,
  handlePriceRangeChange,
  selectedProviders,
  handleProviderChange
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
      <div>
        <label className="block font-semibold mb-2">Rango de precio:</label>

        {/* Price range slider component */}
        <Range
          step={1}
          min={0}
          max={maxPrice}
          values={priceRange}
          onChange={handlePriceRangeChange}  // Update state on slider change
          renderTrack={({ props, children }) => (
            <div {...props} className="w-full h-2 bg-gray-300 rounded-md relative">
              {/* Dynamic blue range selection */}
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
          )}
          renderThumb={({ props, isDragged }) => (
            <div
              {...props}
              className={`w-4 h-4 ${
                isDragged ? "bg-blue-700" : "bg-blue-500"
              } rounded-full shadow-md`}
            />
          )}
        />

        {/* Display selected price range */}
        <div className="flex justify-between text-sm mt-2">
          <span>Mín: {formatPrice(priceRange[0])}</span>
          <span>Máx: {formatPrice(priceRange[1])}</span>
        </div>
      </div>

      {/* Provider Filter */}
      <div className="mt-4">
        <label className="block font-semibold mb-2">Proveedores:</label>
        {PROVIDER_OPTIONS.map((provider) => (
          <div key={provider.key} className="flex items-center mb-2">
            <input
              type="checkbox"
              id={provider.key}
              checked={selectedProviders.includes(provider.key)}
              onChange={() => handleProviderChange(provider.key)}
              className="mr-2"
            />
            <label htmlFor={provider.key} className="text-sm">
              {provider.label}
            </label>
          </div>
        ))}
      </div>

      {/* Message when no products are available */}
      {maxPrice === 0 && (
        <p className="text-sm text-gray-600 mt-4">No hay productos disponibles para filtrar.</p>
      )}
    </aside>
  );
};

export default FilterPanel;
