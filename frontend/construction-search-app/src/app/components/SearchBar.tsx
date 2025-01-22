type SearchBarProps = {
  query: string;  // The current search query
  setQuery: (query: string) => void;  // Function to update the search query
  handleSearch: () => void;  // Function to handle the search button click
  handleKeyDown: (e: React.KeyboardEvent<HTMLInputElement>) => void;  // Function to handle Enter key press
};

/**
 * Component to render a search bar
 * 
 * @param query The current search query
 * @param setQuery Function to update the search query
 * @param handleSearch Function to handle the search button click
 * @param handleKeyDown Function to handle the Enter key press in the search bar
 */
const SearchBar: React.FC<SearchBarProps> = ({ query, setQuery, handleSearch, handleKeyDown }) => {
  return (
    <div className="top-0 z-10 bg-gray-100 p-4 shadow-md mb-4 rounded-md">
      <div className="flex gap-4">
        <input
          type="text"
          placeholder="Buscar productos..."
          value={query}
          onChange={(e) => setQuery(e.target.value)}  // Update query state when typing
          onKeyDown={handleKeyDown}  // Trigger search on Enter key press
          className="border p-2 flex-1 rounded-md"
        />
        <button
          onClick={handleSearch}  // Trigger search on button click
          className="bg-blue-500 text-white px-4 py-2 rounded-md"
        >
          Buscar
        </button>
      </div>
    </div>
  );
};

export default SearchBar;
