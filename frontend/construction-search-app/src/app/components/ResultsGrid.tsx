import React from "react";
import ResultCard from "./ResultCard";

// Type definition for a single product
type Product = {
  name: string;
  store: string;
  url: string;
  image_url: string;
  current_price: number;
  original_price: number | null;
  description: string;
};

type ResultsGridProps = {
  filteredProducts: Product[];  // Filtered list of products to be displayed
  searchClicked: boolean;  // Track if search has been initiated
  formatPrice: (price: number) => string;  // Function to format prices
  pendingStores?: string[];  // List of stores that are still processing
  completedStores?: string[];  // List of stores that have returned results
  loading?: boolean;  // Indicates if results are still loading
  error?: string | null;  // Error message if fetching fails
  searching: boolean;
};

const ResultsGrid: React.FC<ResultsGridProps> = ({
  filteredProducts,
  searchClicked,
  formatPrice,
  pendingStores = [],
  completedStores = [],
  loading,
  error,
  searching,
}) => {
  // Show initial message before any search
  if (!searchClicked && filteredProducts.length === 0) {
    return (
      <div className="text-center py-8 text-gray-600">
        <p>Usa la barra de búsqueda para encontrar productos.</p>
      </div>
    );
  }

  // Show no results message only after search is complete and all stores have been processed
  if (searchClicked && !searching && filteredProducts.length === 0 && pendingStores.length === 0 && completedStores.length > 0) {
    return (
      <div className="text-center py-8 text-gray-600">
        <p>No se encontraron resultados para tu búsqueda.</p>
      </div>
    );
  }

  return (
    <div className="h-[70vh] overflow-y-scroll grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 2xl:grid-cols-4 gap-4 p-4 bg-gray-100 rounded-md shadow-md">
      {(searching || pendingStores.length > 0) && (
        <p className="text-lg font-semibold text-center text-blue-500 col-span-full mx-auto px-4">
          Buscando productos... Por favor, espere.
        </p>
      )}

      {error && (
        <p className="text-lg font-semibold text-center text-red-500 col-span-full mx-auto px-4">
          Error: {error}
        </p>
      )}

      {filteredProducts.map((product, index) => (
        <ResultCard key={index} product={product} formatPrice={formatPrice} />
      ))}

      {/* Show pending and completed stores status */}
      {pendingStores.length > 0 && (
        <p className="text-sm text-yellow-500 col-span-full text-center">
          Procesando resultados de: {pendingStores.join(", ")}
        </p>
      )}

      {completedStores.length > 0 && filteredProducts.length > 0 && (
        <p className="text-sm text-green-500 col-span-full text-center">
          Resultados disponibles de: {completedStores.join(", ")}
        </p>
      )}
    </div>
  );
};

export default ResultsGrid;
