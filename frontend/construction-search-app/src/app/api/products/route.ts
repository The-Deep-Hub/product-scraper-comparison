// No Parameters: This endpoint doesn't accept query parameters (page, query, etc.).
// Response: Simply sends the mockProducts.json content as a JSON array.
// Minimal Logic: The backend does no processing—it's just delivering data.

// Importing Next.js utilities for handling server-side responses
import { NextResponse } from "next/server";

// Importing the mock data to simulate API response data
// This file contains the pre-defined product data in JSON format
import mockData from "../../mockProducts.json";

// The GET function serves as the handler for HTTP GET requests to the endpoint
export async function GET() {
  // Returning the mock data as a JSON response using NextResponse
  // This ensures that the response is formatted correctly for the frontend
  return NextResponse.json(mockData);
}
