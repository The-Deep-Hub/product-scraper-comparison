import React from 'react';
import SearchConfig from './SearchConfig';

interface SearchBarProps {
  query: string;
  setQuery: (query: string) => void;
  handleSearch: () => void;
  handleKeyDown: (e: React.KeyboardEvent) => void;
  searching: boolean;
  searchStores: string[];
  onStoreChange: (store: string) => void;
  productsPerStore: number;
  onProductsPerStoreChange: (value: number) => void;
}

/**
 * Component to render a search bar
 * 
 * @param query The current search query
 * @param setQuery Function to update the search query
 * @param handleSearch Function to handle the search button click
 * @param handleKeyDown Function to handle the Enter key press in the search bar
 * @param searching Indicates if a search is in progress
 * @param searchStores Array of selected stores
 * @param onStoreChange Function to handle store change
 * @param productsPerStore Number of products per store
 * @param onProductsPerStoreChange Function to handle products per store change
 */
const SearchBar: React.FC<SearchBarProps> = ({
  query,
  setQuery,
  handleSearch,
  handleKeyDown,
  searching,
  searchStores,
  onStoreChange,
  productsPerStore,
  onProductsPerStoreChange,
}) => {
  return (
    <div className="flex gap-2 mb-4">
      <input
        type="text"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder="Buscar productos..."
        className="flex-1 p-2 border rounded-lg focus:outline-none focus:border-blue-500"
        disabled={searching}
      />
      <button
        onClick={handleSearch}
        disabled={searching}
        className="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 disabled:bg-blue-300"
      >
        Buscar
      </button>
      <SearchConfig
        selectedStores={searchStores}
        onStoreChange={onStoreChange}
        productsPerStore={productsPerStore}
        onProductsPerStoreChange={onProductsPerStoreChange}
      />
    </div>
  );
};

export default SearchBar;
