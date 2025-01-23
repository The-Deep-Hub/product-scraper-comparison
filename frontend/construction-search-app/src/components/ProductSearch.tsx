import { useState } from 'react';
import { useSearchResults } from '../hooks/useSearchResults';

const AVAILABLE_STORES = ['leroy', 'bauhaus', 'bricodepot'];

export const ProductSearch = () => {
  const [query, setQuery] = useState('');
  const { searchProducts, results, isLoading, error, status, pendingStores } = useSearchResults();

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault();
    if (query.trim()) {
      searchProducts({
        query: query.trim(),
        stores: AVAILABLE_STORES,
        numProducts: 10,
      });
    }
  };

  return (
    <div className="max-w-6xl mx-auto p-4">
      <form onSubmit={handleSearch} className="mb-8">
        <div className="flex gap-4">
          <input
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search for products..."
            className="flex-1 p-2 border rounded"
          />
          <button
            type="submit"
            disabled={isLoading || !query.trim()}
            className="px-4 py-2 bg-blue-500 text-white rounded disabled:bg-gray-300"
          >
            {isLoading ? 'Searching...' : 'Search'}
          </button>
        </div>
      </form>

      {error && (
        <div className="p-4 mb-4 bg-red-100 text-red-700 rounded">
          {error}
        </div>
      )}

      {status === 'searching' && (
        <div className="p-4 mb-4 bg-yellow-100 text-yellow-700 rounded">
          Searching... Results will appear as they become available.
        </div>
      )}

      {Object.entries(results).map(([store, products]) => (
        <div key={store} className="mb-8">
          <h2 className="text-2xl font-bold mb-4 capitalize">{store}</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {products.map((product) => (
              <div key={product.id} className="border rounded p-4">
                <img
                  src={product.urls.image}
                  alt={product.name}
                  className="w-full h-48 object-contain mb-4"
                />
                <h3 className="font-semibold mb-2">{product.name}</h3>
                <p className="text-gray-600 mb-2">{product.description}</p>
                <div className="flex justify-between items-center">
                  <span className="text-lg font-bold">€{product.price.current}</span>
                  <a
                    href={product.urls.product}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-blue-500 hover:underline"
                  >
                    View Details
                  </a>
                </div>
              </div>
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}; 