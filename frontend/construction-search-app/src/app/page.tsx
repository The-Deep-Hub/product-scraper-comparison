"use client";

import { useState } from "react";
import ResultsGrid from "./components/ResultsGrid";
import FilterPanel from "./components/FiltersPanel";
import SearchBar from "./components/SearchBar";

// Type definition for Product, ensuring consistency with the API response
type Product = {
  name: string; // Product name
  store: string; // Store name (e.g., "leroy", "bricodepot")
  url: string; // URL to the product page
  image_url: string; // URL of the product image
  current_price: number; // Current price (in EUR)
  original_price: number | null; // Original price (null if no discount)
  description: string; // Product description (fallback "No description available" if missing)
};

// Define the API endpoint and default stores for search
const API_BASE_URL = "http://localhost:8080";
const DEFAULT_STORES = ["leroy", "bauhaus", "bricodepot"];

export default function Home() {
  // State for managing product data
  const [products, setProducts] = useState<Product[]>([]); // Holds fetched product data
  const [filteredProducts, setFilteredProducts] = useState<Product[]>([]); // Products filtered by query and price
  const [query, setQuery] = useState<string>(""); // Search query input by user
  const [priceRange, setPriceRange] = useState<[number, number]>([0, 100]); // Selected price range
  const [error, setError] = useState<string | null>(null); // Error message state
  const [loading, setLoading] = useState<boolean>(false); // Loading indicator state
  const [maxPrice, setMaxPrice] = useState<number>(100); // Maximum price from fetched products
  const [searchClicked, setSearchClicked] = useState<boolean>(false); // State to track search action

  // Function to initiate product search via the API
  const handleSearch = async () => {
    setSearchClicked(true);
    setError(null);
    setLoading(true);
    setProducts([]); // Clear previous results

    try {
      // Send search request to API
      const response = await fetch(`${API_BASE_URL}/search`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          query,
          stores: DEFAULT_STORES,
          num_products: 10, // Limiting to 10 products per store
        }),
      });

      if (!response.ok) throw new Error("Failed to initiate search");

      const { task_id } = await response.json(); // Extract task ID from response
      await pollForResults(task_id); // Start polling for results
    } catch (err) {
      setError("Error retrieving products. Please try again.");
      console.error("Search error:", err);
    } finally {
      setLoading(false);
    }
  };

  // Function to poll the API for search results using the task ID
  const pollForResults = async (taskId: string) => {
    const pollingInterval = 3000; // 3 seconds between polls
    const timeout = 60000; // Stop polling after 1 minute
    let elapsedTime = 0;

    while (elapsedTime < timeout) {
      try {
        const response = await fetch(`${API_BASE_URL}/task/${taskId}`);
        if (!response.ok) throw new Error("Failed to retrieve search results");

        const taskData = await response.json();

        if (taskData.status === "completed") {
          // Convert API response to frontend Product format
          const formattedProducts = Object.entries(taskData.stores).flatMap(([store, items]: any) =>
            items.map((product: any) => ({
              name: product.name,
              store: store, // Use the store name from API response
              url: product.urls.product,
              image_url: product.urls.image,
              current_price: product.price.current,
              original_price: product.price.original ?? null,
              description: product.description ?? "No description available",
            }))
          );

          setProducts(formattedProducts);
          setFilteredProducts(formattedProducts);

          const maxPrice = Math.max(...formattedProducts.map((p) => p.current_price));
          setPriceRange([0, maxPrice]);
          setMaxPrice(maxPrice);

          return; // Exit polling loop
        } else if (taskData.status === "failed") {
          setError("Search failed. Please try again.");
          console.error("Search failed:", taskData.error);
          return;
        } else {
          // Show pending store status
          console.log("Pending stores:", taskData.pending_stores);
        }
      } catch (err) {
        console.error("Polling error:", err);
        setError("Error retrieving search results.");
        return;
      }

      await new Promise((resolve) => setTimeout(resolve, pollingInterval));
      elapsedTime += pollingInterval;
    }

    setError("Search timed out. Please try again.");
  };

  // Function to filter displayed products based on search query and price range
  const filterProducts = (query: string, range: [number, number]) => {
    const filtered = products.filter(
      (product) =>
        product.name.toLowerCase().includes(query.toLowerCase()) &&
        product.current_price >= range[0] &&
        product.current_price <= range[1]
    );
    setFilteredProducts(filtered);
  };

  // Handle price range updates from the slider
  const handlePriceRangeChange = (values: number[]) => {
    const updatedRange: [number, number] = [values[0], values[1]];
    setPriceRange(updatedRange);
    filterProducts(query, updatedRange);
  };

  // Format price display
  const formatPrice = (price: number) =>
    new Intl.NumberFormat("es-ES", {
      style: "currency",
      currency: "EUR",
    }).format(price);

  if (loading) return <p>Searching for products...</p>;
  if (error) return <p className="text-red-500">{error}</p>;

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
        <SearchBar
          query={query}
          setQuery={setQuery}
          handleSearch={handleSearch}
          handleKeyDown={(e) => e.key === "Enter" && handleSearch()}
        />

        {/* Results Grid */}
        <ResultsGrid filteredProducts={filteredProducts} searchClicked={searchClicked} formatPrice={formatPrice} />
      </main>
    </div>
  );
}
