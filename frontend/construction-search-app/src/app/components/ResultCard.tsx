// ResultCard.tsx
import React from "react";

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

// Props type definition for the ResultCard component
type ResultCardProps = {
  product: Product;
  formatPrice: (price: number) => string;
};

// Component to render individual product cards
const ResultCard: React.FC<ResultCardProps> = ({ product, formatPrice }) => {
  return (
    <div className="resultCard border p-4 rounded-md shadow-sm flex flex-col justify-between">
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
        {product.original_price ? (
          <div>
            <span className="line-through text-gray-500">
              {formatPrice(product.original_price)}
            </span>{" "}
            <span className="font-bold text-green-600">
              {formatPrice(product.current_price)}
            </span>
          </div>
        ) : (
          <p className="font-bold text-lg">{formatPrice(product.current_price)}</p>
        )}
        <p className="text-sm text-gray-600">
          {product.description || "No description available"}
        </p>
      </div>
      <button
        className="mt-4 bg-blue-500 text-white py-2 px-4 rounded-md"
        onClick={() => window.open(product.url, "_blank")}
      >
        Purchase on Site
      </button>
    </div>
  );
};

export default ResultCard;
