# Product Search API Documentation

## Overview

This documentation provides details on how to interact with the Product Search API to retrieve product listings from multiple stores. The API allows searching for products, checking search status, and processing search results.

---

## **1. Search for Products**

### **Endpoint:**
```
POST http://localhost:8080/search
```

### **Request Headers:**
```json
{
  "Content-Type": "application/json"
}
```

### **Request Body Parameters:**
| Parameter    | Type      | Description                                           | Default |
|--------------|-----------|-------------------------------------------------------|---------|
| query        | `string`   | The search keyword                                   | Required |
| stores       | `string[]` | List of stores to search (e.g., `leroy`, `bauhaus`)   | Required |
| num_products | `number`   | Number of products to return per store               | 100     |

### **Example Request:**
```sh
curl -X POST "http://localhost:8080/search" -H "Content-Type: application/json" \
-d '{"query":"pintura verde","stores":["leroy","bauhaus","bricodepot"],"num_products": 10}'
```

### **Response Example:**
```json
{
  "task_id": "fe261c20-e05c-4ef3-9315-bc157d686922",
  "status": "processing",
  "message": "Created 3 tasks for processing. Check status using the task ID."
}
```

---

## **2. Check Search Task Status**

### **Endpoint:**
```
GET http://localhost:8080/task/{task_id}
```

### **Example Request:**
```sh
curl http://localhost:8080/task/fe261c20-e05c-4ef3-9315-bc157d686922 | python -m json.tool
```

### **Possible Response Scenarios:**

#### **a. Processing Response:**
```json
{
  "status": "processing",
  "stores": null,
  "pending_stores": ["leroy", "bauhaus"],
  "error": null
}
```

#### **b. Completed Response:**
```json
{
  "status": "completed",
  "stores": {
    "bricodepot": [
      {
        "id": "9d493da4-6bf3-4962-8424-f8a0832def11",
        "name": "Pintura Verde Manzana Restora Profesional 8 L",
        "price": {
          "current": 22.39,
          "original": null
        },
        "urls": {
          "product": "https://www.bricodepot.es/product/129412",
          "image": "https://assets.bricodepot.es/image/upload/content/asset_images/8422466495565.webp"
        }
      }
    ]
  },
  "pending_stores": null,
  "error": null
}
```

#### **c. Error Response:**
```json
{
  "status": "failed",
  "stores": null,
  "pending_stores": null,
  "error": "Internal server error"
}
```

---

## **3. Integration Workflow**

1. **Initiate a search request** using the `/search` endpoint and store the `task_id` from the response.
2. **Poll the task status** using the `/task/{task_id}` endpoint at regular intervals.
3. **Handle responses:**
   - If the status is `processing`, continue polling.
   - If the status is `completed`, process the received product data.
   - If the status is `failed`, log or display the error message.
4. **Display the product results** to the user in the frontend application.

### **Example Flow:**
```sh
# Step 1: Initiate search
curl -X POST "http://localhost:8080/search" -H "Content-Type: application/json" \
-d '{"query":"martillo blanco","stores":["leroy","bauhaus"],"num_products": 10}'

# Step 2: Check task status
curl http://localhost:8080/task/fe261c20-e05c-4ef3-9315-bc157d686922 | python -m json.tool

# Repeat step 2 until status is "completed"
```

---

## **4. Data Structure**

Each product result includes the following fields:

| Field         | Type      | Description                                      |
|---------------|-----------|--------------------------------------------------|
| id            | `string`   | Unique identifier for the product               |
| name          | `string`   | Product name                                    |
| description   | `string`   | Product description                             |
| price.current | `number`   | Current product price                           |
| price.original| `number`   | Original price if a discount is available       |
| urls.product  | `string`   | URL to the product page                         |
| urls.image    | `string`   | URL to the product image                        |

---

## **5. Notes**

- Ensure to check the `status` field in the task response before processing the product data.
- The API is designed to handle concurrent store queries and returns results progressively.
- When all stores are processed, `pending_stores` will be set to `null`.

