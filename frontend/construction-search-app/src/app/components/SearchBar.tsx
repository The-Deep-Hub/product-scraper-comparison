type SearchBarProps = {
  query: string;  // The current search query
  setQuery: (query: string) => void;  // Function to update the search query
  handleSearch: () => void;  // Function to handle the search button click
  handleKeyDown: (e: React.KeyboardEvent<HTMLInputElement>) => void;  // Function to handle Enter key press
  searching: boolean; // Indicates if a search is in progress
};

/**
 * Component to render a search bar
 * 
 * @param query The current search query
 * @param setQuery Function to update the search query
 * @param handleSearch Function to handle the search button click
 * @param handleKeyDown Function to handle the Enter key press in the search bar
 * @param searching Indicates if a search is in progress
 */
const SearchBar: React.FC<SearchBarProps> = ({ query, setQuery, handleSearch, handleKeyDown, searching }) => {
  return (
    <div className="top-0 z-10 bg-gray-100 p-4 shadow-md mb-4 rounded-md">
      <div className="flex gap-4">
        <input
          type="text"
          placeholder="Buscar productos..."
          value={query}
          onChange={(e) => setQuery(e.target.value.trimStart())}  // Update query state when typing
          onKeyDown={handleKeyDown}  // Trigger search on Enter key press
          className="border p-2 flex-1 rounded-md"
          disabled={searching}  // Disable input when searching
        />
        <button
          onClick={handleSearch}  // Trigger search on button click
          disabled={!query.trim() || searching}  // Prevent empty and multiple searches
          className={`px-4 py-2 rounded-md ${searching ? "bg-gray-400 cursor-not-allowed" : "bg-blue-500 text-white"}`}
        >
          {searching ? "Buscando..." : "Buscar"}
        </button>
      </div>
    </div>
  );
};

export default SearchBar;
