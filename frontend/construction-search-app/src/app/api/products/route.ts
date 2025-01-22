// Importing Next.js utilities for handling server-side responses
import { NextResponse } from "next/server";

// Importing mock data for fallback (optional testing)
import mockData from "../../mockProducts.json";

// Define the real API base URL (should be set in an environment variable)
const API_BASE_URL = process.env.API_BASE_URL || "http://localhost:8080";

// The GET function serves as the handler for HTTP GET requests to the endpoint
export async function GET() {
  // Check if mock mode is enabled via environment variables
  const useMockData = process.env.USE_MOCK_DATA === "true";

  if (useMockData) {
    // Return mock data for testing purposes
    return NextResponse.json(mockData);
  }

  try {
    // Forward the request to the real API
    const response = await fetch(`${API_BASE_URL}/search`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        query: "default", // Placeholder, will be replaced with real search queries
        stores: ["leroy", "bauhaus", "bricodepot"],
        num_products: 5,
      }),
    });

    if (!response.ok) {
      throw new Error("Failed to fetch products from the real API");
    }

    const data = await response.json();
    return NextResponse.json(data);
  } catch (error) {
    console.error("API proxy error:", error);
    return NextResponse.json({ error: "Failed to fetch data" }, { status: 500 });
  }
}
