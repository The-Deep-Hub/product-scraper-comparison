type SearchBarProps = {
  // The current search query
  query: string;
  // Function to update the search query
  setQuery: (query: string) => void;
  // Function to handle the search button click
  handleSearch: () => void;
  // Function to handle the Enter key press in the search bar
  handleKeyDown: (e: React.KeyboardEvent<HTMLInputElement>) => void;
};

/**
 * Component to render a search bar
 * @param query The current search query
 * @param setQuery Function to update the search query
 * @param handleSearch Function to handle the search button click
 * @param handleKeyDown Function to handle the Enter key press in the search bar
 * @returns A search bar component
 */
const SearchBar: React.FC<SearchBarProps> = ({ query, setQuery, handleSearch, handleKeyDown }) => {
  return (
    <div className="top-0 z-10 bg-gray-100 p-4 shadow-md mb-4 rounded-md">
      <div className="flex gap-4">
        <input
          type="text"
          // Placeholder for the search bar
          placeholder="Buscar productos..."
          // The current search query
          value={query}
          // Function to update the search query
          onChange={(e) => setQuery(e.target.value)}
          // Function to handle the Enter key press in the search bar
          onKeyDown={handleKeyDown}
          // Styling for the search bar
          className="border p-2 flex-1 rounded-md"
        />
        <button
          // Function to handle the search button click
          onClick={handleSearch}
          // Styling for the search button
          className="bg-blue-500 text-white px-4 py-2 rounded-md"
        >
          Buscar
        </button>
      </div>
    </div>
  );
};

export default SearchBar;

