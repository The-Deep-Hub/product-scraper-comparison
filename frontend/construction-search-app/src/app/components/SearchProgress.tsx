import React, { useEffect, useState } from "react";

const SearchProgress: React.FC<{ duration: number }> = ({ duration }) => {
  const [progress, setProgress] = useState(0);

  useEffect(() => {
    let interval = setInterval(() => {
      setProgress((prev) => (prev < 100 ? prev + (100 / (duration / 1000)) : 100));
    }, 1000);

    return () => clearInterval(interval);
  }, [duration]);

  return (
    <div className="fixed top-20 right-6 flex items-center space-x-2">
      <div className="relative w-12 h-12">
        <svg className="absolute inset-0" viewBox="0 0 36 36">
          <path
            className="text-gray-300"
            strokeWidth="4"
            fill="none"
            d="M18 2a16 16 0 1 1 0 32 16 16 0 1 1 0-32"
          />
          <path
            className="text-blue-500"
            strokeWidth="4"
            fill="none"
            strokeDasharray="100,100"
            strokeDashoffset={`${100 - progress}`}
            d="M18 2a16 16 0 1 1 0 32 16 16 0 1 1 0-32"
          />
        </svg>
      </div>
      <span className="text-sm font-medium text-gray-700">{Math.round(progress)}% Searching...</span>
    </div>
  );
};

export default SearchProgress;

