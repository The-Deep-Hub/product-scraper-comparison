"use client";

import { useEffect, useState } from "react";
import ResultsGrid from "./components/ResultsGrid";
import FilterPanel from "./components/FiltersPanel";
import SearchBar from "./components/SearchBar";
import MessageDialog from "./components/MessageDialog";
import ProviderStatusDialog from "./components/ProviderStatusDialog";
import SearchProgress from "./components/SearchProgress";

// Type definition for Product, ensuring consistency with the API response
type Product = {
  name: string;  // Product name
  store: string;  // Store name (e.g., "leroy", "bricodepot")
  url: string;  // URL to the product page
  image_url: string;  // URL of the product image
  current_price: number;  // Current price (in EUR)
  original_price: number | null;  // Original price (null if no discount)
  description: string;  // Product description (fallback to "No description available" if missing)
};

// Define the API endpoint and default stores for search
const API_BASE_URL = "http://localhost:8080";
const DEFAULT_STORES = ["leroy", "bauhaus", "bricodepot"];

export default function Home() {
  // State for managing product data
  const [products, setProducts] = useState<Product[]>([]);  // Holds fetched product data
  const [filteredProducts, setFilteredProducts] = useState<Product[]>([]);  // Products filtered by query and price
  const [query, setQuery] = useState<string>("");  // Search query input by user
  const [priceRange, setPriceRange] = useState<[number, number]>([0, 100]);  // Selected price range
  const [error, setError] = useState<string | null>(null);  // Error message state
  const [loading, setLoading] = useState<boolean>(false);  // Loading indicator state
  const [maxPrice, setMaxPrice] = useState<number>(100);  // Maximum price from fetched products
  const [searchClicked, setSearchClicked] = useState<boolean>(false);  // State to track search action
  const [pendingStores, setPendingStores] = useState<string[]>([]);  // Stores still being processed
  const [searching, setSearching] = useState<boolean>(false);  // Prevents multiple searches
  const [stopSearch, setStopSearch] = useState<boolean>(false);  // Stops ongoing search
  const [progress, setProgress] = useState<number>(0);
  const [selectedProviders, setSelectedProviders] = useState<string[]>(DEFAULT_STORES);
  const [lastQuery, setLastQuery] = useState<string>("");  // Store last search query
  const [lastProviders, setLastProviders] = useState<string[]>([]);  // Store last provider selection


  useEffect(() => {
    if (products.length > 0) {
      const highestPrice = Math.max(...products.map((p) => p.current_price));
      if (highestPrice !== maxPrice) {
        setMaxPrice(highestPrice);
        setPriceRange([0, highestPrice]);
      }
    } else {
      setMaxPrice(100);
      setPriceRange([0, 100]);
    }
  }, [products, maxPrice]);



  // Function to initiate product search via the API
  const handleSearch = async () => {
    if (searching) {
      setError("A search is already in progress. Please wait for it to finish.");
      return;
    }

    if (!query.trim()) {
      setError("Please enter a search query.");
      return;
    }

    // Avoid duplicate searches by checking if query and providers are the same
    if (query === lastQuery && selectedProviders.sort().join(",") === lastProviders.sort().join(",")) {
      setError("This search has already been performed.");
      return;
    }

    // Update last search state
    setLastQuery(query);
    setLastProviders(selectedProviders);

    setSearchClicked(true);
    setError(null);
    setLoading(true);
    setSearching(true);  // Block new searches
    setStopSearch(false);  // Reset stop state
    setProducts([]);  // Clear previous results
    setPendingStores(DEFAULT_STORES);  // Assume all stores are pending initially
    setProgress(0);
    setMaxPrice(100);  // Default to a reasonable max value before new search
    setPriceRange([0, 100]); // Default range before search


    try {
      // Send search request to API
      const response = await fetch(`${API_BASE_URL}/search`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          query,
          stores: DEFAULT_STORES,
          num_products: 7,  // Limiting to 5 products per store
        }),
      });

      if (!response.ok) throw new Error("Failed to initiate search");

      const { task_id } = await response.json();  // Extract task ID from response
      await pollForResults(task_id);  // Start polling for results
    } catch (err) {
      setError("Error retrieving products. Please try again.");
      console.error("Search error:", err);
    } finally {
      setLoading(false);
      setSearching(false);  // Allow new searches
    }
  };

  // Function to poll the API for search results using the task ID
  const pollForResults = async (taskId: string) => {
    const pollingInterval = 3000;  // 3 segundos entre cada consulta
    const timeout = 90000;  // Detener después de 1.5 minutos
    let elapsedTime = 0;
    let allProducts: Product[] = [];

    while (elapsedTime < timeout) {
      if (stopSearch) {
        setError("Search was stopped by the user.");
        break;
      }

      try {
        const response = await fetch(`${API_BASE_URL}/task/${taskId}`);
        if (!response.ok) throw new Error("Failed to retrieve search results");

        const taskData = await response.json();

        // Update progress
        const completedStores = DEFAULT_STORES.length - (taskData.pending_stores?.length || 0);
        setProgress(Math.round((completedStores / DEFAULT_STORES.length) * 100));

        if (taskData.stores) {
          // Process and store new products from API response
          const newProducts = Object.entries(taskData.stores).flatMap(([store, items]: any) =>
            items.map((product: any) => ({
              name: product.name,
              store: store,
              url: product.urls.product,
              image_url: product.urls.image,
              current_price: product.price.current,
              original_price: product.price.original ?? null,
              description: product.description && product.description !== "No description available"
                ? product.description
                : product.name, // Use name if no description
            }))
          );

          // Merge products and remove duplicates
          allProducts = [...allProducts, ...newProducts].filter(
            (product, index, self) => index === self.findIndex((p) => p.url === product.url)
          );

          // Sort the products by price (ascending order)
          allProducts.sort((a, b) => a.current_price - b.current_price);

          setProducts(allProducts);
          setFilteredProducts(allProducts);

          // Determine the new maximum price and update states
          if (newProducts.length > 0) {
            // Determine the new maximum price and update states
            const newMaxPrice = Math.max(...allProducts.map((p) => p.current_price), 0);
            setMaxPrice(newMaxPrice);
            setPriceRange([0, newMaxPrice]);
          }

        }


        if (taskData.status === "completed") {
          setPendingStores([]);
          return;  // Exit polling when search is complete
        } else {
          setPendingStores(taskData.pending_stores || []);
        }
      } catch (err) {
        console.error("Polling error:", err);
        setError("Error retrieving search results.");
        setPendingStores([]);
        return;
      }

      await new Promise((resolve) => setTimeout(resolve, pollingInterval));
      elapsedTime += pollingInterval;
    }

    // Display whatever results were found once timeout is reached
    setError("Search time completed. Displaying found results.");
    setPendingStores([]);
    setSearching(false);
  };



  // Function to handle provider checkbox change
  const handleProviderChange = (provider: string) => {
    setSelectedProviders((prevProviders) => {
      const updatedProviders = prevProviders.includes(provider)
        ? prevProviders.filter((p) => p !== provider)  // Remove provider
        : [...prevProviders, provider];  // Add provider

      // Call filtering function after updating selected providers
      filterProducts(query, priceRange, updatedProviders);

      return updatedProviders;
    });
  };


  // Function to filter displayed products based on search query, price range, and selected providers
  const filterProducts = (query: string, range: [number, number], providers: string[]) => {
    const filtered = products.filter(
      (product) =>
        product.name.toLowerCase().includes(query.toLowerCase()) &&
        product.current_price >= range[0] &&
        product.current_price <= range[1] &&
        providers.includes(product.store) // Filter by selected providers
    );
    setFilteredProducts(filtered);
  };

  // Handle price range updates from the slider
  const handlePriceRangeChange = (values: number[]) => {
    setPriceRange([values[0], values[1]]);
    filterProducts(query, [values[0], values[1]], selectedProviders);
  };



  // Format price display
  const formatPrice = (price: number) =>
    new Intl.NumberFormat("es-ES", {
      style: "currency",
      currency: "EUR",
    }).format(price);

  return (
    <div className="flex flex-col md:flex-row gap-4 p-4">
      {/* Filters Panel */}
      <FilterPanel
        maxPrice={maxPrice}
        priceRange={priceRange}
        formatPrice={formatPrice}
        handlePriceRangeChange={handlePriceRangeChange}
        selectedProviders={selectedProviders}
        handleProviderChange={handleProviderChange}
      />

      {/* Main Content */}
      <main className="flex-1">
        <SearchBar
          query={query}
          setQuery={setQuery}
          handleSearch={handleSearch}
          handleKeyDown={(e) => e.key === "Enter" && handleSearch()}
          searching={searching}
        />

        {/* Progress Indicator */}
        {searching && <SearchProgress duration={90000} />}

        {/* Status and Loading Messages as Dialogs */}
        {loading && <MessageDialog message="Retrieving search results, please wait..." type="loading" onClose={() => setLoading(false)} />}
        {error && <MessageDialog message={error} type="error" onClose={() => setError(null)} />}
        {pendingStores.length > 0 && <ProviderStatusDialog pendingStores={pendingStores} completedStores={DEFAULT_STORES.filter((store) => !pendingStores.includes(store))} />}

        {/* Results Grid */}
        <ResultsGrid filteredProducts={filteredProducts} searchClicked={searchClicked} formatPrice={formatPrice} />
      </main>
    </div>
  );
}
