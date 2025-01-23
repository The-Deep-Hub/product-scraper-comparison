import { useState, useEffect } from 'react';

interface SearchParams {
  query: string;
  stores: string[];
  numProducts: number;
}

interface Product {
  id: string;
  name: string;
  description: string;
  price: {
    current: number;
    original: number | null;
  };
  urls: {
    product: string;
    image: string;
  };
}

interface TaskResponse {
  task_id: string;
  status: string;
  message?: string;
}

interface TaskStatus {
  status: 'processing' | 'completed' | 'failed';
  stores: { [key: string]: Product[] };
  pending_stores: string[];
  error: string | null;
}

export const useSearchResults = () => {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [results, setResults] = useState<{ [key: string]: Product[] }>({});
  const [taskId, setTaskId] = useState<string | null>(null);
  const [status, setStatus] = useState<'idle' | 'searching' | 'completed' | 'failed'>('idle');

  const searchProducts = async (params: SearchParams) => {
    try {
      setIsLoading(true);
      setError(null);
      setStatus('searching');
      setResults({});

      // Initial search request
      const response = await fetch(`${process.env.NEXT_PUBLIC_API_URL}/search`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          query: params.query,
          stores: params.stores,
          num_products: params.numProducts,
        }),
      });

      if (!response.ok) {
        throw new Error('Search request failed');
      }

      const data: TaskResponse = await response.json();
      setTaskId(data.task_id);

    } catch (err) {
      setError(err instanceof Error ? err.message : 'An error occurred');
      setStatus('failed');
    } finally {
      setIsLoading(false);
    }
  };

  // Polling effect
  useEffect(() => {
    let pollInterval: NodeJS.Timeout;

    const pollResults = async () => {
      if (!taskId || status !== 'searching') return;

      try {
        const response = await fetch(`${process.env.NEXT_PUBLIC_API_URL}/task/${taskId}`);
        if (!response.ok) {
          throw new Error('Failed to fetch task status');
        }

        const data: TaskStatus = await response.json();

        if (data.error) {
          setError(data.error);
          setStatus('failed');
          return;
        }

        // Update results with any completed store data
        setResults(data.stores || {});

        if (data.status === 'completed' || data.pending_stores.length === 0) {
          setStatus('completed');
        }

      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to fetch results');
        setStatus('failed');
      }
    };

    if (taskId && status === 'searching') {
      // Poll every 2 seconds
      pollInterval = setInterval(pollResults, 2000);
    }

    return () => {
      if (pollInterval) {
        clearInterval(pollInterval);
      }
    };
  }, [taskId, status]);

  return {
    searchProducts,
    results,
    isLoading,
    error,
    status,
    pendingStores: status === 'searching',
  };
}; 