# Scraping Guide for Obramat Product Data

## Objective

To scrape product data from Obramat's API endpoint and extract structured information about products for a specified query. The extracted data will include various fields like product title, price, availability, brand, and more. This guide explains the scraping process, the fields to be extracted, and the expected output structure.

## API Details

### Endpoint
`https://na.search.sensefuel.live/search/53484288-56a5-421b-a049-356b096f9840`

### Headers

```json
{
    "Content-Type": "text/plain",
    "Origin": "https://www.obramat.es",
    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36"
}
```

### Payload Template

```json
{
    "query": "<search_term>",
    "filters": [],
    "pagination": {
        "page": 1,
        "perPage": 20
    },
    "sort": {
        "field": "relevance",
        "direction": "desc"
    }
}
```

## Scraping Workflow

### 1. Set Up HTTP Request

- Use the provided API endpoint and headers.
- Replace `<search_term>` in the payload with the desired product search term (e.g., "radiadores").
- Set the pagination to fetch only the top 20 results for simplicity.

### 2. Send POST Request

- Use a library like Python's requests to send the POST request with the payload and headers.

### 3. Parse the JSON Response

- The API returns a JSON object containing product information.
- Extract the items from the JSON response (data.items.p).

### 4. Extract Relevant Fields

- For each product in the response, map the relevant fields to a structured dictionary.

### 5. Display Data

- Print the extracted data to the console for review.

## Output Fields

### Top-Level Fields

| Field | Description |
|-------|-------------|
| ID | Product-specific ID (id) |
| Global ID | Unique global product identifier (gid) |
| Title | Product name or title (ttl) |
| Price | Primary price (prc) |
| Secondary Price | Additional price field (c_precio secundario) |
| Brand | Brand of the product (brn) |
| Availability | Stock status (avl) |
| Image URL | URL of the primary product image (img) |
| Additional Image URL | URL of additional product image (addilnk) |
| Product URL | Link to the product on the Obramat website (lnk) |
| Packaging Type | Type of packaging (e.g., "Pieza") (c_packaging_type) |
| Packaging Quantity | Quantity in packaging (c_packaging_quantity) |
| New Product | Boolean indicating if the product is new (isNew) |

### Nested Fields (Offers)

| Field | Description |
|-------|-------------|
| Offer Price (with Tax) | Offer price (offers.prcn) |
| Quantity Available | Stock quantity (offers.qty) |
| Delivery Options | Delivery details (offers.c_delivery) |
| Offer Availability | Status of the offer (offers.c_availibility) |
| Offer Label | Label for the offer (e.g., "En stock") (offers.c_label) |

### Statistical Fields

| Field | Description |
|-------|-------------|
| Min Price | Minimum price across vendors (stats.nsaa.mn) |
| Max Price | Maximum price across vendors (stats.nsaa.mx) |

## Example Output

### Product 1:

- **ID**: 10425562
- **Global ID**: 10425562
- **Title**: RADIADOR ALUMINIO FERROLI XIAN 600 6 ELEMENTOS
- **Price**: 55,00
- **Secondary Price**: 9.17
- **Brand**: FERROLI
- **Availability**: in stock
- **Image URL**: https://www.obramat.es/media/catalog/product/1/0/6/6/radiador_aluminio_ferroli_xian_elementos_10425562_picture_01.jpeg?
- **Additional Image URL**: https://www.obramat.es/media/catalog/product/f/9/9/4/radiador_aluminio_ferroli_xian_elementos_10425562_picture_02.jpeg
- **Product URL**: https://www.obramat.es/radiador-de-aluminio-xian-6-elementos-500-mm-10425562.html
- **Packaging Type**: Pieza (BM)
- **Packaging Quantity**: 6.0000
- **Offer Price (with Tax)**: 60
- **Quantity Available**: 54
- **Delivery Options**: 0FREE72HOURSITE_DELIVERY0FREE2HOURONSITE
- **Offer Availability**: En stock
- **Min Price**: 55
- **Max Price**: 55

## Considerations

### Error Handling

- Handle HTTP errors (e.g., status code != 200).
- Handle cases where no products are found.

### Rate Limiting

- Avoid overloading the API by implementing delays or handling rate limits.

### Data Validation

- Ensure all extracted fields have valid data; fallback to "N/A" if not available.

## Next Steps

1. Test the scraper with multiple search terms to ensure robustness.
2. Add functionality to export results to a file (if needed in the future).
3. Monitor for changes in the API structure to update the scraper accordingly.