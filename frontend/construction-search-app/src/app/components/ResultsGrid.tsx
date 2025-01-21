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
  filteredProducts: Product[];
  searchClicked: boolean;
  formatPrice: (price: number) => string;
};

const ResultsGrid: React.FC<ResultsGridProps> = ({ filteredProducts, searchClicked, formatPrice }) => {
  return (
    <div className="h-[70vh] overflow-y-scroll grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xxl:grid-cols-4 gap-4 p-4 bg-gray-100 rounded-md shadow-md">
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
          <ResultCard key={index} product={product} formatPrice={formatPrice} />
        ))
      )}
    </div>
  );
};

export default ResultsGrid;
