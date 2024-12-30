"use client";

import { useState, useEffect } from "react";
import { Range } from "react-range";


// Removed `mockData` import as data is now fetched from `/api/products`.

// Type definition for Product, ensuring consistency with the API response
type Product = {
  name: string; // Product name
  store: string; // Store name (e.g., "leroy", "bricodepot")
  url: string; // URL to the product page
  image_url: string; // URL of the product image
  current_price: {
    amount: number; // Current price
    currency: string; // Currency code (e.g., "EUR")
  };
  original_price: {
    amount: number | null; // Original price (null if no discount)
    currency: string | null; // Currency code (null if no discount)
  };
  discount: number | null; // Discount percentage (null if no discount)
  description: string; // Product description
  metadata: {
    sku: string | null; // Unique product identifier (optional, null if unavailable)
    source: string; // Data source (e.g., "leroymerlin", "bricodepot")
  };
};

export default function Home() {
  // State for managing products fetched from the backend
  const [products, setProducts] = useState<Product[]>([]); // All products fetched from API
  const [filteredProducts, setFilteredProducts] = useState<Product[]>([]); // Products filtered by search query and price range
  const [query, setQuery] = useState<string>(""); // User's search query
  const [priceRange, setPriceRange] = useState<[number, number]>([0, 100]); // Selected price range
  const [error, setError] = useState<string | null>(null); // Error message state
  const [loading, setLoading] = useState<boolean>(true); // Loading state to manage fetch status
  const [maxPrice, setMaxPrice] = useState<number>(100); // Maximum price from fetched products
  const [searchClicked, setSearchClicked] = useState<boolean>(false); // **Fixed: State to track search action**

  // Fetch data from API on component mount
  useEffect(() => {
    const fetchProducts = async () => {
      try {
        const response = await fetch("/api/products"); // GET request to fetch product data
        if (!response.ok) throw new Error("Failed to fetch products"); // Handle non-200 status codes

        const data: Product[] = await response.json(); // Parse JSON response
        setProducts(data); // Store fetched products
        setFilteredProducts(data); // Initialize filtered products with all fetched data

        const maxPrice = Math.max(...data.map((p) => p.current_price.amount)); // Calculate maximum price
        setPriceRange([0, maxPrice]); // Set initial price range
        setMaxPrice(maxPrice); // Set maximum price for slider
      } catch (err) {
        setError(err instanceof Error ? err.message : "An unknown error occurred"); // Set error message for display
      } finally {
        setLoading(false); // Stop loading indicator
      }
    };

    fetchProducts();
  }, []); // Run only once on mount


  // Function to filter products based on search query and price range
  const filterProducts = (query: string, range: [number, number]) => {
    const filtered = products.filter(
      (product) =>
        product.name.toLowerCase().includes(query.toLowerCase()) &&
        product.current_price.amount >= range[0] &&
        product.current_price.amount <= range[1]
    );
    setFilteredProducts(filtered); // Comment: Update filtered products
  };

  // Handle search button click
  const handleSearch = () => {
    setSearchClicked(true); // **Fixed: Update searchClicked state**
    filterProducts(query, priceRange); // Filter products based on current search and range
  };

  // Handle Enter key press in the search bar
  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") handleSearch(); // Comment: Trigger search on Enter key press
  };

  // Handle changes in the price range slider
  const handlePriceRangeChange = (values: number[]) => {
    const updatedRange: [number, number] = [values[0], values[1]];
    setPriceRange(updatedRange); // Comment: Update selected price range
    filterProducts(query, updatedRange); // Comment: Re-filter products in real-time
  };

  // Format price for display in currency format
  const formatPrice = (price: number) =>
    new Intl.NumberFormat("es-ES", {
      style: "currency",
      currency: "EUR",
    }).format(price); // Comment: Format price as per Spanish locale

  if (loading) return <p>Loading...</p>; // Comment: Display loading message while data is being fetched
  if (error) return <p>Error: {error}</p>; // Comment: Display error message if fetching fails


  // Component rendering
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
            onChange={handlePriceRangeChange}
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
                className={`w-4 h-4 ${
                  isDragged ? "bg-blue-700" : "bg-blue-500"
                } rounded-full shadow-md`}
              />
            )}
          />
          {/* Selected Price Range */}
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
              onKeyDown={handleKeyDown}
              className="border p-2 flex-1 rounded-md"
            />
            <button
              onClick={handleSearch}
              className="bg-blue-500 text-white px-4 py-2 rounded-md"
            >
              Buscar
            </button>
          </div>
        </div>

        {/* Results Grid */}
        <div className="h-[70vh] overflow-y-scroll grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 p-4 bg-gray-100 rounded-md shadow-md">
          {loading ? (
            <p className="text-center text-gray-500">Cargando productos...</p>
          ) : error ? (
            <p className="text-red-500 text-center">{error}</p>
          ) : searchClicked && filteredProducts.length === 0 ? (
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
                    src={product.image_url}
                    alt={product.name}
                    className="w-full h-32 object-cover rounded-md mb-4"
                  />
                  <h3 className="font-bold">{product.name}</h3>
                  <a
                    href={product.url}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-blue-500 hover:underline"
                  >
                    {product.store}
                  </a>
                  {product.discount ? (
                    <div>
                      <span className="line-through text-gray-500">
                        {formatPrice(product.original_price.amount!)}
                      </span>{" "}
                      <span className="font-bold text-green-600">
                        {formatPrice(product.current_price.amount)}
                      </span>
                    </div>
                  ) : (
                    <p className="font-bold text-lg">
                      {formatPrice(product.current_price.amount)}
                    </p>
                  )}
                  <p className="text-sm text-gray-600">{product.description}</p>
                </div>
                <button
                  className="mt-4 bg-blue-500 text-white py-2 px-4 rounded-md"
                  onClick={() => window.open(product.url, "_blank")}
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