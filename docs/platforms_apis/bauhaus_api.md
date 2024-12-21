# Documentation: Extracting and Using JSON-LD Content for Web Scraping

## Overview

Many modern websites provide structured data in JSON-LD (Linked Data) format embedded within their HTML. This structured data is primarily designed for SEO purposes, allowing search engines to understand the content of the page. As a scraper, you can leverage JSON-LD to extract key product details efficiently without relying on parsing dynamic or heavily nested HTML.

## JSON-LD Content Description

### Basic Product Details

| Field | Description |
|-------|-------------|
| Name | The product name. |
| Image | URL of the product image. |
| URL | Direct link to the product page. |
| SKU | Stock Keeping Unit identifier. |
| Brand | Brand name or manufacturer. |

### Offer Information

| Field | Description |
|-------|-------------|
| Price | The current price of the product. |
| Currency | The currency in which the price is displayed (e.g., EUR). |
| Availability | Product availability status (e.g., InStock, OutOfStock). |
| Condition | Condition of the product (e.g., NewCondition). |
| Seller | Organization or entity selling the product. |

## Example JSON-LD

```json
{
  "@context": "https://schema.org/",
  "@type": "Product",
  "name": "Encimera de madera maciza",
  "image": "https://media.cdn.bauhaus/m/488801/15.jpg",
  "description": "",
  "url": "https://www.bauhaus.es/tableros-de-madera-maciza/exclusivholz-encimera-de-madera-maciza/p/24516543",
  "sku": "24516543",
  "mpn": "24516543",
  "brand": { "@type": "Thing", "name": "Exclusivholz" },
  "offers": {
    "@type": "Offer",
    "url": "https://www.bauhaus.es/tableros-de-madera-maciza/exclusivholz-encimera-de-madera-maciza/p/24516543",
    "priceCurrency": "EUR",
    "price": "149.00",
    "itemCondition": "https://schema.org/NewCondition",
    "availability": "https://schema.org/InStock",
    "seller": { "@type": "Organization", "name": "BAUHAUS" }
  }
}
```

## Step-by-Step Procedure to Extract JSON-LD

### 1. Identify JSON-LD Content

1. Open the webpage in a browser.
2. Inspect the HTML source (Ctrl+U or right-click -> "View Page Source").
3. Search for `<script type="application/ld+json">`.

### 2. Extract JSON-LD Content Programmatically

Use a Python script with libraries like BeautifulSoup and json:

```python
from bs4 import BeautifulSoup
import json
import requests

# Fetch the webpage
url = "https://www.bauhaus.es/tableros-de-madera-maciza/exclusivholz-encimera-de-madera-maciza/p/24516543"
response = requests.get(url)
soup = BeautifulSoup(response.text, 'html.parser')

# Extract JSON-LD content
json_ld_tag = soup.find('script', type='application/ld+json')
if json_ld_tag:
    json_ld_data = json.loads(json_ld_tag.string)
    print(json_ld_data)
```

### 3. Parse and Save Extracted Data

Convert the JSON data into structured output like a CSV, database, or dictionary:

```python
import csv

# Extract relevant fields
product_data = {
    'name': json_ld_data.get('name'),
    'brand': json_ld_data.get('brand', {}).get('name'),
    'price': json_ld_data.get('offers', {}).get('price'),
    'currency': json_ld_data.get('offers', {}).get('priceCurrency'),
    'availability': json_ld_data.get('offers', {}).get('availability'),
    'url': json_ld_data.get('url')
}

# Save to CSV
with open('products.csv', 'w', newline='', encoding='utf-8') as file:
    writer = csv.DictWriter(file, fieldnames=product_data.keys())
    writer.writeheader()
    writer.writerow(product_data)
```

## Best Practices

### Handle Missing Fields Gracefully

- Not all fields may be present in every JSON-LD block. Use `.get()` to avoid key errors.

### Verify Data Accuracy

- Compare the extracted JSON-LD with rendered HTML content to ensure consistency.

### Rate-Limiting and Throttling

- Respect the website's terms of service and avoid excessive requests.

### Monitor Updates

- Websites frequently update their structure. Regularly test your scraping script to ensure continued functionality.

### Combine with Other Data

- If JSON-LD lacks details (e.g., strikethrough prices), supplement it with parsed HTML or API data.

## Use Cases for JSON-LD Content

1. **Price Monitoring**: Track price changes across e-commerce sites.
2. **SEO Analysis**: Evaluate structured data implementation for competitors.
3. **Market Research**: Extract product catalogs, brands, and offers.

By leveraging JSON-LD content, you can simplify the scraping process while ensuring access to structured and clean data.