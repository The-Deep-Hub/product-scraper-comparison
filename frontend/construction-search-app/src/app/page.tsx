"use client";

import { useState, useEffect } from "react";
import { Range } from "react-range";
import mockData from "./mockData.json";

// Define the product type for mock data
type Product = {
  productName: string;
  providerName: string;
  providerURL: string;
  imageURL: string;
  price: number;
  discount?: number;
  originalPrice?: number;
  description: string;
  purchaseLink: string;
};

export default function Home() {
  // State for the search query
  const [query, setQuery] = useState("");
  // State for the price range filter
  const [priceRange, setPriceRange] = useState<[number, number]>([0, 100]);
  // State for the filtered products to display
  const [filteredProducts, setFilteredProducts] = useState<Product[]>([]);
  // State to store the maximum price from the mock data
  const [maxPrice, setMaxPrice] = useState(100);
  // State to check if the search button has been clicked
  const [searchClicked, setSearchClicked] = useState(false);

  // On component mount, calculate the maximum price from the mock data
  useEffect(() => {
    const highestPrice = Math.ceil(
      Math.max(...mockData.map((product) => product.price))
    );
    setMaxPrice(highestPrice);
    setPriceRange([0, highestPrice]);
  }, []);

  // Function to filter products based on the search query and price range
  const filterProducts = (query: string, range: [number, number]) => {
    const filtered = mockData.filter(
      (product) =>
        product.productName.toLowerCase().includes(query.toLowerCase()) &&
        product.price >= range[0] &&
        product.price <= range[1]
    );
    setFilteredProducts(filtered);
  };

  // Handle search button click
  const handleSearch = () => {
    setSearchClicked(true);
    filterProducts(query, priceRange);
  };

  // Handle Enter key press in the search bar
  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") {
      handleSearch();
    }
  };

  // Handle changes in the price range slider
  const handlePriceRangeChange = (values: number[]) => {
    const updatedRange = [values[0], values[1]] as [number, number];
    setPriceRange(updatedRange);
    filterProducts(query, updatedRange); // Update results in real-time
  };

  // Format price for display in currency format
  const formatPrice = (price: number) =>
    new Intl.NumberFormat("es-ES", {
      style: "currency",
      currency: "EUR",
    }).format(price);

  return (
    <div className="flex flex-col md:flex-row gap-4 p-4">
      {/* Filters Panel */}
      <aside className="responsive top-4 w-full md:w-1/4 bg-gray-100 p-4 rounded-md shadow-md">
        <h2 className="text-lg font-bold mb-4">Filtros</h2>
        <div>
          <label className="block mb-2">Rango de precio:</label>
          {/* Price Range Slider */}
          <Range
            step={1}
            min={0}
            max={maxPrice}
            values={priceRange}
            onChange={handlePriceRangeChange} // Update results in real-time
            renderTrack={({ props, children }) => (
              <div
                {...props}
                className="w-full h-2 bg-gray-300 rounded-md relative"
              >
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
                className={`w-4 h-4 ${isDragged ? "bg-blue-700" : "bg-blue-500"
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
      </aside>

      {/* Main Content */}
      <main className="flex-1">
        {/* Search Bar */}
        <div className="top-0 z-10 bg-gray-100 p-4 shadow-md mb-4 rounded-md">
          <div className="flex gap-4">
            <input
              type="text"
              placeholder="Buscar productos..."
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={handleKeyDown} // Handle Enter key
              className="border p-2 flex-1 rounded-md"
            />
            <button
              onClick={handleSearch} // Trigger search
              className="bg-blue-500 text-white px-4 py-2 rounded-md"
            >
              Buscar
            </button>
          </div>
        </div>

        {/* Results Grid */}
        <div className="h-[70vh] overflow-y-scroll grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 p-4 bg-gray-100 rounded-md shadow-md">
          {searchClicked && filteredProducts.length === 0 ? (
            <p className="text-lg font-semibold text-center text-red-500 col-span-full mx-auto px-4">
              No se encontraron resultados para tu búsqueda.
            </p>
          ) : !searchClicked ? (
            <p className="text-lg font-semibold text-center text-gray-600 col-span-full mx-auto px-4">
              Usa la barra de búsqueda para encontrar productos.
            </p>
          ) : (
            filteredProducts.map((product, index) => (
              <div
                key={index}
                className="resultCard border p-4 rounded-md shadow-sm flex flex-col justify-between"
              >
                <div>
                  <img
                    src={product.imageURL}
                    alt={product.productName}
                    className="w-full h-32 object-cover rounded-md mb-4"
                  />
                  <h3 className="font-bold">{product.productName}</h3>
                  <a
                    href={product.providerURL}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-blue-500 hover:underline"
                  >
                    {product.providerName}
                  </a>
                  {product.discount ? (
                    <div>
                      <span className="line-through text-gray-500">
                        {formatPrice(product.originalPrice!)}
                      </span>{" "}
                      <span className="font-bold text-green-600">
                        {formatPrice(product.price)}
                      </span>
                    </div>
                  ) : (
                    <p className="font-bold text-lg">
                      {formatPrice(product.price)}
                    </p>
                  )}
                  <p className="text-sm text-gray-600">{product.description}</p>
                </div>
                <button
                  className="mt-4 bg-blue-500 text-white py-2 px-4 rounded-md"
                  onClick={() => window.open(product.purchaseLink, "_blank")}
                >
                  Purchase on Site
                </button>
              </div>
            ))
          )}
        </div>
      </main>
    </div>
  );
}
