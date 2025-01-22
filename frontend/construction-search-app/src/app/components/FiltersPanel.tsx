import { Range } from "react-range";

type FilterPanelProps = {
  maxPrice: number;
  priceRange: [number, number];
  formatPrice: (price: number) => string;
  handlePriceRangeChange: (values: number[]) => void;
};

const FilterPanel: React.FC<FilterPanelProps> = ({ maxPrice, priceRange, formatPrice, handlePriceRangeChange }) => {
  return (
    <aside className="responsive top-4 w-full md:w-1/4 bg-gray-100 p-4 rounded-md shadow-md">
      <h2 className="text-lg font-bold mb-4">Filtros</h2>
      <div>
        <label className="block mb-2">Rango de precio:</label>
        <Range
          step={1}
          min={0}
          max={maxPrice}
          values={priceRange}
          onChange={handlePriceRangeChange}
          renderTrack={({ props, children }) => (
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
          )}
          renderThumb={({ props, isDragged }) => (
            <div
              {...props}
              className={`w-4 h-4 ${isDragged ? "bg-blue-700" : "bg-blue-500"} rounded-full shadow-md`}
            />
          )}
        />
        <div className="flex justify-between text-sm mt-2">
          <span>Mín: {formatPrice(priceRange[0])}</span>
          <span>Máx: {formatPrice(priceRange[1])}</span>
        </div>
      </div>
    </aside>
  );
};

export default FilterPanel;
