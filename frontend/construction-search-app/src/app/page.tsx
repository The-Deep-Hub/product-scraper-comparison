"use client";

import { useState, useEffect } from "react";
import { Range } from "react-range";
import ResultCard from "./components/ResultCard";
import ResultsGrid from "./components/ResultsGrid";
import FilterPanel from "./components/FiltersPanel";


// Removed `mockData` import as data is now fetched from `/api/products`.

// Type definition for Product, ensuring consistency with the API response
type Product = {
  name: string; // Product name
  store: string; // Store name (e.g., "leroy", "bricodepot")
  url: string; // URL to the product page
  image_url: string; // URL of the product image
  current_price: number; // Current price (in EUR)
  original_price: number | null; // Original price (null if no discount)
  description: string; // Product description (e.g., "No description available"). Might return "No description available" as missing description. Title could be a substitute to handle missing data. 
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
  const [searchClicked, setSearchClicked] = useState<boolean>(false); // State to track search action

  // Fetch data from API on component mount
  useEffect(() => {
    const fetchProducts = async () => {
      try {
        const response = await fetch("/api/products"); // GET request to fetch product data
        if (!response.ok) throw new Error("Failed to fetch products"); // Handle non-200 status codes

        const data: Product[] = await response.json(); // Parse JSON response
        setProducts(data); // Store fetched products
        setFilteredProducts(data); // Initialize filtered products with all fetched data

        const maxPrice = Math.max(...data.map((p) => p.current_price)); // Calculate maximum price
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
        product.current_price >= range[0] &&
        product.current_price <= range[1]
    );
    setFilteredProducts(filtered); // Update filtered products
  };

  // Handle search button click
  const handleSearch = () => {
    setSearchClicked(true); // Update searchClicked state
    filterProducts(query, priceRange); // Filter products based on current search and range
  };

  // Handle Enter key press in the search bar
  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") handleSearch(); // Trigger search on Enter key press
  };

  // Handle changes in the price range slider
  const handlePriceRangeChange = (values: number[]) => {
    const updatedRange: [number, number] = [values[0], values[1]];
    setPriceRange(updatedRange); // Update selected price range
    filterProducts(query, updatedRange); // Re-filter products in real-time
  };

  // Format price for display in currency format
  const formatPrice = (price: number) =>
    new Intl.NumberFormat("es-ES", {
      style: "currency",
      currency: "EUR",
    }).format(price); // Format price as per Spanish locale

  if (loading) return <p>Loading...</p>; // Display loading message while data is being fetched
  if (error) return <p>Error: {error}</p>; // Display error message if fetching fails

  // Component rendering
  return (
    <div className="flex flex-col md:flex-row gap-4 p-4">
      {/* Filters Panel */}
      <FilterPanel
        maxPrice={maxPrice}
        priceRange={priceRange}
        formatPrice={formatPrice}
        handlePriceRangeChange={handlePriceRangeChange}
      />

      {/* Main Content */}
      <main className="flex-1">
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
            <button onClick={handleSearch} className="bg-blue-500 text-white px-4 py-2 rounded-md">
              Buscar
            </button>
          </div>
        </div>

        {/* Results Grid */}
        <ResultsGrid
          filteredProducts={filteredProducts}
          searchClicked={searchClicked}
          formatPrice={formatPrice}
        />

      </main>
    </div>
  );
}