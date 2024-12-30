# API Endpoint: `/api/products`

## **Purpose**
The frontend sends a request to the backend via this endpoint to retrieve product search results. The backend responds with either:
1. **Search Results**: A JSON array containing product details.
2. **Error Response**: If no results are found or an error occurs.

## **Endpoint Details**
- **URL**: `/api/products`
- **Method**: `GET`

---

## **Request**
The endpoint does not require any request body or query parameters for this mock version.

---

## **Response**
### **Success Response**
Returns an array of product objects in the following structure:
```json
[
  {
    "name": "string",                 // Product name
    "store": "string",                // Store name (e.g., "leroy", "bricodepot")
    "url": "string",                  // URL to the product page
    "image_url": "string",            // URL of the product image
    "current_price": {
      "amount": "number",             // Current price
      "currency": "string"            // Currency code (e.g., "EUR")
    },
    "original_price": {
      "amount": "number|null",        // Original price (null if no discount)
      "currency": "string|null"       // Currency code (null if no discount)
    },
    "discount": "number|null",        // Discount percentage (null if no discount)
    "description": "string",          // Product description (e.g., "No description available")
    "metadata": {
      "sku": "string|null",           // Unique product identifier (optional, null if unavailable)
      "source": "string"              // Data source (e.g., "leroymerlin", "bricodepot")
    }
  }
]
```

### **Example Success Response**

```json
[
  {
    "name": "Martillo Clásico",
    "store": "leroymerlin",
    "url": "https://www.leroymerlin.es/product/martillo-clasico",
    "image_url": "https://via.placeholder.com/150",
    "current_price": {
      "amount": 10.99,
      "currency": "EUR"
    },
    "original_price": {
      "amount": null,
      "currency": null
    },
    "discount": null,
    "description": "Martillo tradicional con mango de madera.",
    "metadata": {
      "sku": "93281797",
      "source": "leroymerlin"
    }
  }
]
```

---

## **Error Response**
If no results are found or an error occurs:
```json
{
  "status": "error",
  "message": "No products found."
}
```

# **API Usage**

## **Frontend Behavior**
- Sends a `GET` request to `/api/products`.
- Receives:
  - **Search Results**: A JSON array containing product details.
  - **Error Message**: If no results are found or an error occurs.

---

## **Backend Behavior**
- Processes the incoming request.
- Returns the response in the expected format, either:
  - **Success**: An array of product objects.
  - **Error**: An error message indicating no results were found.

---

## **Example Use Cases**
1. **Populate Search Results**:
   - Use the returned product data to populate the search results grid in the frontend.
2. **Handle No Results**:
   - Display an error message if no results are found.

---

## **Testing**
- **Tool**: Use Postman or a similar API testing tool.
- **Steps**:
  1. Run the application locally (`http://localhost:3000`).
  2. Create a `GET` request to `http://localhost:3000/api/products`.
  3. Verify that the response:
     - Matches the expected structure.
     - Displays either a valid product list or an appropriate error message.
