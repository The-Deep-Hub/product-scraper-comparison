# Comprehensive Guide to Scraping Bricodepot Website Using Algolia API

## 1. Introduction

Bricodepot's website uses Algolia's search service to provide data for its product listings. This makes it possible to extract structured product data directly via the Algolia API endpoints. In this document, we will explore the methods to scrape this data, evaluate the pros and cons, provide a backup alternative, and document the detailed response fields for reference.

## 2. Methods of Scraping the Website

### 2.1 Using the Algolia API

**Steps:**

#### Inspect the Network Tab:

1. Open the browser's developer tools (F12) and go to the "Network" tab.
2. Perform a search query on the website (e.g., "sillas blancas").
3. Identify the Algolia API request. It usually has the format:
   `https://[application-id]-dsn.algolia.net/1/indexes/*/queries`

#### Extract Required Headers:

Identify the `x-algolia-application-id` and `x-algolia-api-key` from the request headers.

**Example:**

- `x-algolia-application-id`: JGGZJ7UXAX
- `x-algolia-api-key`: Y2I1ZDczZDMxNjkwMjZjNzNlMTcwMTdjZDZjYzdiNjgzNzE2OWZlOGMyNzAyOGJiZTNkNDNmNGUxY2M4MTM4NXRhZ0ZpbHRlcnM9JnZhbGlkVW50aWw9MTczNDg5ODM1Mw==

#### Craft the Payload:

Use the payload structure identified in the network request.

**Example payload for searching "sillas blancas":**

```json
{
    "requests": [
        {
            "indexName": "pro_ES_products",
            "params": "query=sillas blancas&hitsPerPage=60&page=0"
        }
    ]
}
```

#### Send the Request:

Use Python's requests library to send a POST request to the Algolia API.

**Example code:**

```python
import requests

url = "https://jggzj7uxax-dsn.algolia.net/1/indexes/*/queries"

headers = {
    "x-algolia-application-id": "JGGZJ7UXAX",
    "x-algolia-api-key": "Y2I1ZDczZDMxNjkwMjZjNzNlMTcwMTdjZDZjYzdiNjgzNzE2OWZlOGMyNzAyOGJiZTNkNDNmNGUxY2M4MTM4NXRhZ0ZpbHRlcnM9JnZhbGlkVW50aWw9MTczNDg5ODM1Mw==",
    "Content-Type": "application/json",
}

payload = {
    "requests": [
        {
            "indexName": "pro_ES_products",
            "params": "query=sillas blancas&hitsPerPage=60&page=0",
        }
    ]
}

response = requests.post(url, json=payload, headers=headers)
print(response.json())
```

#### Parse the Response:

The API returns a structured JSON response. Extract the desired fields (e.g., product name, price, URL, etc.).

### 2.2 Web Scraping as a Backup Alternative

If the Algolia API becomes inaccessible (e.g., the API key is revoked), a backup method involves traditional web scraping using libraries like BeautifulSoup or Selenium.

**Steps:**

#### Inspect the HTML Structure:

1. Analyze the website's DOM to locate product listing elements.
2. Identify classes or IDs associated with product details (e.g., name, price, image).

#### Use Python Libraries:

Use requests to fetch the HTML content or Selenium for dynamic content.

**Example code using BeautifulSoup:**

```python
from bs4 import BeautifulSoup
import requests

url = "https://www.bricodepot.es/categorias/sillas-blancas"
headers = {
    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/110.0.0.0 Safari/537.36"
}

response = requests.get(url, headers=headers)
soup = BeautifulSoup(response.text, "html.parser")

for product in soup.select(".product-card"):
    name = product.select_one(".product-name").text.strip()
    price = product.select_one(".product-price").text.strip()
    print(f"Name: {name}, Price: {price}")
```

## 3. Pros and Cons of Each Method

### 3.1 Using the Algolia API

**Pros:**
- Structured and clean data
- Faster and more efficient
- No need to parse HTML
- Minimal risk of getting blocked if headers mimic browser behavior

**Cons:**
- Relies on the availability of the API key
- Limited to data exposed by the API

### 3.2 Web Scraping

**Pros:**
- Can extract any visible content on the website
- Works even if the API key is revoked

**Cons:**
- Slower due to HTML parsing and possible use of Selenium
- Higher risk of IP blocks or captchas
- Requires maintenance if the website's structure changes

## 4. Backup Plan

If neither the Algolia API nor direct web scraping is viable:

### Use a Proxy Service:
- Services like ScrapFly or Bright Data can bypass restrictions by rotating IPs and solving captchas.

### Monitor API Key Changes:
- Automate the process of capturing updated API keys by monitoring network requests periodically.

### Evaluate Alternative Sources:
- Look for other public endpoints or third-party APIs that provide similar data.

## 5. Detailed Response Fields from the Algolia API

The following table summarizes the fields in the API response:

| Field | Description |
|-------|-------------|
| name | Product name. |
| price_front.price | Current price of the product. |
| price_front.was_price | Original price before discount (if applicable). |
| categories.level0 | Top-level category. |
| categories.level1 | Subcategory. |
| categories.level2 | Further breakdown of category. |
| categories.level3 | Most detailed category level. |
| sku | Stock-keeping unit (SKU). |
| identifier_ean | European Article Number (EAN) of the product. |
| image_url | URL to the product's image. |
| url | URL to the product's webpage. |
| stock_qty | Number of items available in stock. |
| in_stock | Boolean indicating if the product is in stock. |
| campaign_nombre | Campaign name (if part of a promotional campaign). |
| visibility | Visibility of the product (e.g., Catalog, Search). |

## 6. Conclusion

Scraping the Bricodepot website using the Algolia API is a reliable and efficient method to gather structured product data. However, it is crucial to have a robust backup plan, such as traditional web scraping, to ensure continuity. Regular monitoring of API behavior and network requests is recommended to maintain access.

By understanding the detailed API response fields, developers can build flexible and scalable data extraction pipelines for various use cases.