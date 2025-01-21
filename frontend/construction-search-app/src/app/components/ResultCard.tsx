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
    <div className="resultCard border p-4 rounded-md shadow-sm flex flex-col justify-between max-w-[300px] max-h-[500px] w-full h-full mx-auto gap-2">
      {/* Product Image */}
      <div className="w-full h-40 flex items-center justify-center rounded-md bg-white">
        <img
          src={product.image_url}
          alt={product.name}
          className="max-w-full max-h-full object-contain rounded-md mb-4"
        />
      </div>

      {/* Product Information */}
      <div className="flex-1 flex flex-col justify-between">
        <div>
          <h3 className="font-bold text-lg truncate">{product.name}</h3>
          <a
            href={product.url}
            target="_blank"
            rel="noopener noreferrer"
            className="text-blue-500 hover:underline"
          >
            {product.store}
          </a>
          {/* Price information */}
          {product.original_price ? (
            <div className="mt-2">
              <span className="line-through text-gray-500">
                {formatPrice(product.original_price)}
              </span>{" "}
              <span className="font-bold text-green-600">
                {formatPrice(product.current_price)}
              </span>
            </div>
          ) : (
            <p className="font-bold text-lg mt-2">{formatPrice(product.current_price)}</p>
          )}
          {/* Product description */}
          <p className="text-sm text-gray-600 mt-2 line-clamp-3">
            {product.description || "No description available"}
          </p>
        </div>

        {/* Purchase button */}
        <button
          className="mt-4 bg-blue-500 text-white py-2 px-4 rounded-md w-full"
          onClick={() => window.open(product.url, "_blank")}
        >
          Purchase on Site
        </button>
      </div>
    </div>
  );
};

export default ResultCard;
