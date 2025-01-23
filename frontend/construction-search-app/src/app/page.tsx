"use client";

import { useEffect, useState } from "react";
import ResultsGrid from "./components/ResultsGrid";
import FilterPanel from "./components/FiltersPanel";
import SearchBar from "./components/SearchBar";
import MessageDialog from "./components/MessageDialog";
import ProviderStatusDialog from "./components/ProviderStatusDialog";
import SearchProgress from "./components/SearchProgress";
import SearchConfig from './components/SearchConfig';

// Type definition for Product, ensuring consistency with the API response
type Product = {
  name: string;  // Product name
  store: string;  // Store name (e.g., "leroy", "bricodepot")
  url: string;  // URL to the product page
  image_url: string;  // URL of the product image
  current_price: number;  // Current price (in EUR)
  original_price: number | null;  // Original price (null if no discount)
  description: string;  // Product description
};

// API Response Types
interface ApiProduct {
  name: string;
  urls: {
    product: string;
    image: string;
  };
  price: {
    current: number;
    original: number | null;
  };
  description?: string;
}

interface ApiResponse {
  status: 'completed' | 'processing';
  stores?: { [key: string]: ApiProduct[] };
  task_id?: string;
  pending_stores?: string[];
}

// Define the API endpoint and default stores for search
const API_BASE_URL = "http://localhost:8080";
const DEFAULT_STORES = ["leroy", "bauhaus", "bricodepot"];

// Helper function to determine store from URL
const getStoreFromUrl = (url: string): string => {
  if (url.includes('leroymerlin.es')) return 'leroy';
  if (url.includes('bauhaus.es')) return 'bauhaus';
  if (url.includes('bricodepot.es')) return 'bricodepot';
  return 'unknown';
};

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

  // State for search configuration
  const [searchStores, setSearchStores] = useState<string[]>(DEFAULT_STORES);
  const [productsPerStore, setProductsPerStore] = useState<number>(7);
  // State for filtering results
  const [filterStores, setFilterStores] = useState<string[]>(DEFAULT_STORES);

  // New state for offers filter and sort order
  const [showOnlyOffers, setShowOnlyOffers] = useState(false);
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('asc');  // Default to ascending

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

  // Handle store selection for search
  const handleSearchStoreChange = (store: string) => {
    setSearchStores(prev => 
      prev.includes(store) 
        ? prev.filter(s => s !== store)
        : [...prev, store]
    );
  };

  // Handle products per store change
  const handleProductsPerStoreChange = (value: number) => {
    setProductsPerStore(value);
  };

  // Handle store selection for filtering
  const handleFilterStoreChange = (store: string) => {
    const updated = filterStores.includes(store)
      ? filterStores.filter(s => s !== store)
      : [...filterStores, store];
    
    setFilterStores(updated);
    filterProducts(query, priceRange, updated, showOnlyOffers, sortOrder);
  };

  // Function to filter displayed products
  const filterProducts = (
    searchQuery: string, 
    range: [number, number], 
    stores: string[],
    offersOnly: boolean,
    currentSortOrder: 'asc' | 'desc'
  ) => {
    if (!products.length) return;
    
    if (stores.length === 0) {
      setFilteredProducts([]);
      return;
    }
    
    let filtered = products.filter(product => {
      const matchesStore = stores.includes(product.store);
      const matchesPrice = product.current_price >= range[0] && product.current_price <= range[1];
      const matchesOffers = !offersOnly || (offersOnly && product.original_price !== null);
      
      return matchesStore && matchesPrice && matchesOffers;
    });

    // Apply sorting if selected
    if (currentSortOrder) {
      filtered.sort((a, b) => {
        if (currentSortOrder === 'asc') {
          return a.current_price - b.current_price;
        } else {
          return b.current_price - a.current_price;
        }
      });
    }

    setFilteredProducts(filtered);
  };

  useEffect(() => {
    if (products.length > 0) {
      filterProducts(query, priceRange, filterStores, showOnlyOffers, sortOrder);
    }
  }, [products, query, priceRange, filterStores, showOnlyOffers, sortOrder]);

  // Update handleSearch to include productsPerStore
  const handleSearch = async () => {
    if (searching) {
      setError("A search is already in progress. Please wait for it to finish.");
      return;
    }

    if (!query.trim()) {
      setError("Please enter a search query.");
      return;
    }

    setSearchClicked(true);
    setError(null);
    setLoading(true);
    setSearching(true);
    setStopSearch(false);
    setProducts([]);
    setPendingStores(searchStores);
    setProgress(0);
    setMaxPrice(100);
    setPriceRange([0, 100]);
    // Initialize filter stores with search stores
    setFilterStores(searchStores);

    try {
      const response = await fetch(`${API_BASE_URL}/search`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          query,
          stores: searchStores,
          num_products: productsPerStore,
        }),
      });

      if (!response.ok) throw new Error("Failed to initiate search");

      const data: ApiResponse = await response.json();
      
      // Check if we got immediate results or a task ID
      if (data.status === "completed" && data.stores) {
        // Process immediate results
        const newProducts: Product[] = Object.entries(data.stores).flatMap(([_, items]) =>
          items.map((product: ApiProduct) => ({
            name: product.name,
            store: getStoreFromUrl(product.urls.product),  // Use URL to determine store
            url: product.urls.product,
            image_url: product.urls.image,
            current_price: product.price.current,
            original_price: product.price.original,
            description: product.description || product.name,
          }))
        );

        // Sort products by price
        newProducts.sort((a: Product, b: Product) => a.current_price - b.current_price);
        
        setProducts(newProducts);
        setFilteredProducts(newProducts);
        
        // Update price range
        if (newProducts.length > 0) {
          const newMaxPrice = Math.max(...newProducts.map((p: Product) => p.current_price));
          setMaxPrice(newMaxPrice);
          setPriceRange([0, newMaxPrice]);
        }
        
        setProgress(100);
        setPendingStores([]);
      } else if (data.task_id) {
        // Start polling for results if we got a task ID
        await pollForResults(data.task_id);
      } else {
        throw new Error("Invalid response from server");
      }
    } catch (err) {
      setError("Error retrieving products. Please try again.");
      console.error("Search error:", err);
    } finally {
      setLoading(false);
      setSearching(false);
    }
  };

  // Function to poll the API for search results using the task ID
  const pollForResults = async (taskId: string) => {
    const pollingInterval = 3000;
    const timeout = 90000;
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

        const taskData: ApiResponse = await response.json();

        // Update progress based on completed stores
        const completedStores = DEFAULT_STORES.length - (taskData.pending_stores?.length || 0);
        setProgress(Math.round((completedStores / DEFAULT_STORES.length) * 100));

        if (taskData.stores) {
          // Process and store new products from API response
          const newProducts: Product[] = Object.entries(taskData.stores).flatMap(([_, items]) =>
            items.map((product: ApiProduct) => ({
              name: product.name,
              store: getStoreFromUrl(product.urls.product),  // Determine store from URL
              url: product.urls.product,
              image_url: product.urls.image,
              current_price: product.price.current,
              original_price: product.price.original,
              description: product.description || product.name,
            }))
          );

          // Merge products and remove duplicates
          allProducts = [...allProducts, ...newProducts].filter(
            (product, index, self) => 
              index === self.findIndex((p) => p.url === product.url)
          );

          // Sort the products by price (ascending order)
          allProducts.sort((a, b) => a.current_price - b.current_price);

          setProducts(allProducts);
          setFilteredProducts(allProducts);

          // Update price range if we have products
          if (allProducts.length > 0) {
            const newMaxPrice = Math.max(...allProducts.map((p) => p.current_price));
            setMaxPrice(newMaxPrice);
            setPriceRange([0, newMaxPrice]);
          }
        }

        // Check if search is complete
        if (taskData.status === "completed" || taskData.pending_stores?.length === 0) {
          setPendingStores([]);
          setProgress(100);
          return;  // Exit polling when search is complete
        } else {
          setPendingStores(taskData.pending_stores || []);
        }

        // If all stores returned results immediately (cache hit), exit polling
        if (completedStores === DEFAULT_STORES.length) {
          setPendingStores([]);
          setProgress(100);
          return;
        }
      } catch (err) {
        console.error("Polling error:", err);
        setError("Error retrieving search results.");
        setPendingStores([]);
        return;
      }

      // Only wait if we need to continue polling
      if (elapsedTime < timeout) {
        await new Promise((resolve) => setTimeout(resolve, pollingInterval));
        elapsedTime += pollingInterval;
      }
    }

    // Display whatever results were found once timeout is reached
    if (elapsedTime >= timeout) {
      setError("Search time completed. Displaying found results.");
    }
    setPendingStores([]);
    setSearching(false);
  };

  // Handle price range updates from the slider
  const handlePriceRangeChange = (values: number[]) => {
    const newRange: [number, number] = [values[0], values[1]];
    setPriceRange(newRange);
    filterProducts(query, newRange, filterStores, showOnlyOffers, sortOrder);
  };

  // Format price display
  const formatPrice = (price: number) =>
    new Intl.NumberFormat("es-ES", {
      style: "currency",
      currency: "EUR",
    }).format(price);

  // Toggle offers filter
  const toggleOffers = () => {
    const newShowOnlyOffers = !showOnlyOffers;
    setShowOnlyOffers(newShowOnlyOffers);
    // Immediately filter products with the new value
    filterProducts(query, priceRange, filterStores, newShowOnlyOffers, sortOrder);
  };

  // Handle sort order change
  const handleSortChange = (order: 'asc' | 'desc') => {
    setSortOrder(order);
    filterProducts(query, priceRange, filterStores, showOnlyOffers, order);
  };

  return (
    <div className="flex flex-col md:flex-row gap-4 p-4">
      {/* Filters Panel */}
      <FilterPanel
        maxPrice={maxPrice}
        priceRange={priceRange}
        formatPrice={formatPrice}
        handlePriceRangeChange={handlePriceRangeChange}
        selectedProviders={filterStores}
        handleProviderChange={handleFilterStoreChange}
        showOnlyOffers={showOnlyOffers}
        toggleOffers={toggleOffers}
        sortOrder={sortOrder}
        handleSortChange={handleSortChange}
      />

      {/* Main Content */}
      <main className="flex-1">
        <SearchBar
          query={query}
          setQuery={setQuery}
          handleSearch={handleSearch}
          handleKeyDown={(e) => e.key === "Enter" && handleSearch()}
          searching={searching}
          searchStores={searchStores}
          onStoreChange={handleSearchStoreChange}
          productsPerStore={productsPerStore}
          onProductsPerStoreChange={handleProductsPerStoreChange}
        />

        {/* Progress Indicator */}
        {searching && <SearchProgress duration={90000} />}

        {/* Status and Loading Messages as Dialogs */}
        {loading && <MessageDialog message="Retrieving search results, please wait..." type="loading" onClose={() => setLoading(false)} />}
        {error && <MessageDialog message={error} type="error" onClose={() => setError(null)} />}
        {pendingStores.length > 0 && <ProviderStatusDialog pendingStores={pendingStores} completedStores={DEFAULT_STORES.filter((store) => !pendingStores.includes(store))} />}

        {/* Results Grid */}
        <ResultsGrid filteredProducts={filteredProducts} searchClicked={searchClicked} formatPrice={formatPrice} searching={false} />
      </main>
    </div>
  );
}
