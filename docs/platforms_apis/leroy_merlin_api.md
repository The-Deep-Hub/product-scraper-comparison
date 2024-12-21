# Documenting the Leroy Merlin Product Scraping Process

## Characteristics of Leroy Merlin Data

The product data for Leroy Merlin is embedded within the HTML as JSON objects inside script tags. Specifically, these JSON objects are structured with detailed information about each product.

## Fields Returned by Leroy Merlin JSON

| Field | Description |
|-------|-------------|
| brand | The brand of the product. |
| name | The product name. |
| sku | Stock keeping unit, a unique identifier for each product. |
| price | Current price of the product (includes VAT). |
| original_price | The original price if the product is discounted. |
| discount_rate | Discount rate (if applicable). |
| seller | Name of the seller (e.g., Leroy Merlin or third-party sellers). |
| seller_type | Indicates the type of seller (1P for Leroy Merlin, 3P for third-party). |
| rating | Customer rating of the product (if available). |
| total_offer_count | Number of offers for the product. |
| url | Relative URL for the product page on Leroy Merlin's website. |
| image_url | Constructed URL for the product image. |
| **Offer Details** | |
| offer_details.offer_id | Unique identifier for the specific offer. |
| offer_details.seller_id | ID of the seller for the offer. |
| offer_details.seller_name | Name of the seller providing the offer. |
| offer_details.unitprice_ati | Price per unit including taxes. |
| offer_details.unitprice_tf | Price per unit excluding taxes. |
| position | Position of the product in the search results. |
| product_is_sponsored | Indicates if the product is sponsored. |
| marketing_tag | Marketing tags for the product (e.g., NONE for no tags). |
| type | The type of item (e.g., product). |
| commercial_animation | Animations or promotions applied to the product. |

## Pros and Cons of Leroy Merlin API Integration

### Pros

1. **Detailed Data**: The JSON objects provide comprehensive details about each product, including offers, pricing, and seller information.
2. **Availability in HTML**: The data is directly embedded in the HTML, eliminating the need for separate API authentication.
3. **Structured Format**: The JSON structure is consistent and easy to parse programmatically.
4. **Additional Details**: Includes marketing tags, ratings, and seller composition, which are useful for analytics.

### Cons

1. **Dynamic Content**: The JSON data is embedded in script tags, making it harder to locate compared to traditional API endpoints.
2. **HTML Parsing Overhead**: Requires additional steps to parse the HTML before extracting JSON data.
3. **Rate Limits**: Scraping may trigger rate limits or blocking mechanisms if performed too aggressively.
4. **Dependencies**: Relies on consistent HTML structure, which may break if the website's structure changes.

## Approach to Scraping Leroy Merlin

### Steps to Extract Product Data

#### 1. HTTP Request

Make an HTTP GET request to the search result URL using headers to mimic a browser.

**Example Headers:**

```python
headers = {
    'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36',
    'Accept': 'text/html,application/xhtml+xml',
    'Accept-Language': 'es-ES,es;q=0.9',
    'Referer': 'https://www.leroymerlin.es/'
}
```

#### 2. Parse HTML

- Use a library like BeautifulSoup to parse the HTML content.
- Locate all script tags with application/json type and dataTms class.

#### 3. Extract and Parse JSON

- Extract JSON content from the script tags and load it using Python's json library.
- Identify the product list structure and iterate over each product to extract relevant fields.

#### 4. Construct Image URLs

Leroy Merlin does not provide direct image URLs. Use a base image URL (e.g., https://media.adeo.com/media/) and append the SKU or specific keys to construct the image link.

#### 5. Error Handling

- Log errors during parsing and handle JSON decoding issues gracefully.

#### 6. Store or Display Results

- Print the results to the console or save them to a file for further use.

## Example Script

```python
from bs4 import BeautifulSoup
import requests
import json
import logging

def parse_product_data(product):
    """Parse individual product data"""
    try:
        base_image_url = "https://media.adeo.com/media/"
        return {
            'name': product.get('name'),
            'brand': product.get('brand'),
            'sku': product.get('sku'),
            'price': product.get('offer', {}).get('price'),
            'original_price': product.get('displayed_price'),
            'discount_rate': product.get('discount_rate'),
            'seller': product.get('seller'),
            'rating': product.get('rating'),
            'url': f"https://www.leroymerlin.es{product.get('url')}",
            'image_url': f"{base_image_url}{product.get('sku')}/media.jpg"
        }
    except Exception as e:
        logging.error(f"Error parsing product: {e}")
        return None

def scrape_leroy_merlin_products(url):
    headers = {
        'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36',
        'Accept': 'text/html,application/xhtml+xml',
        'Accept-Language': 'es-ES,es;q=0.9'
    }
    response = requests.get(url, headers=headers)
    soup = BeautifulSoup(response.text, 'html.parser')
    data_scripts = soup.find_all('script', {'type': 'application/json', 'class': 'dataTms'})
    all_products = []
    for script in data_scripts:
        try:
            data = json.loads(script.string)
            if isinstance(data, list) and len(data) > 0:
                products_data = data[0].get('value', [])
                all_products.extend(products_data)
        except Exception as e:
            logging.error(f"Error processing script tag: {e}")
    parsed_products = [parse_product_data(product) for product in all_products if product]
    return parsed_products

if __name__ == "__main__":
    url = "https://www.leroymerlin.es/search?q=radiador"
    products = scrape_leroy_merlin_products(url)
    for product in products:
        print(product)
```

## Output Fields

The script outputs a list of dictionaries with the following fields:

- Product name
- Brand
- SKU
- Price
- Original price
- Discount rate
- Seller
- Rating
- Product URL
- Image URL