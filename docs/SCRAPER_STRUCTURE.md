# Scraper Structure Documentation

This document details the HTML structure and extraction methods used by each scraper in the system.

## Bauhaus Scraper

### Product Card Structure
The Bauhaus scraper extracts product information from two main sources:
1. JSON-LD structured data (primary source)
2. HTML selectors (fallback mechanism)

#### JSON-LD Structure
Location: Found in a `<script type="application/ld+json">` tag that is a sibling to the product card.
```json
{
  "@type": "Product",
  "name": "Product Name",
  "image": ["Image URL"],
  "sku": "Product SKU",
  "offers": {
    "price": "Price",
    "priceCurrency": "EUR"
  }
}
```

#### HTML Selectors (Fallback)
Product Card: `.product-list-item`
- Title: `.product-list-item__title`
- Price: `.product-list-item__price-tag-price`
- Image: `.product-list-item__image img[src]`
- Product URL: `.product-list-item__title a[href]`

### Pagination Structure
- Next Page Link: `.pagination__item--next`
- Page Numbers: `.pagination__item`

## Leroy Merlin Scraper

### Product Card Structure
Main Product Container: `.product-card`

#### Product Information Selectors
- Title: `.product-card__description-title`
- Price Container: `.product-price__amount`
  - Integer Part: `.product-price__integer`
  - Decimal Part: `.product-price__decimal`
- Image: `.product-card__media img[src]`
- Product URL: `.product-card__description-title a[href]`

### Pagination Structure
- Next Page Button: `.pagination__next-page-button`
- Page Numbers: `.pagination__page-number`

## BricoDepot Scraper

### Product Card Structure
Main Product Container: `.product-miniature`

#### Product Information Selectors
- Title: `.product-title`
- Price: `.product-price`
  - Regular Price: `.regular-price`
  - Current Price: `.current-price`
- Image: `.product-thumbnail img[src]`
- Product URL: `.product-thumbnail a[href]`

### Pagination Structure
- Next Page Link: `.page-next`
- Page Numbers: `.page-number`

## Common Extraction Patterns

All scrapers follow these common patterns:
1. First attempt to locate the main product container
2. Extract individual elements using specific selectors
3. Transform and validate the extracted data
4. Handle missing or malformed data gracefully

### Error Handling
All scrapers implement fallback mechanisms:
- Missing images → Log warning and continue
- Invalid prices → Skip product and log error
- Missing titles → Skip product and log error

### URL Construction
- All relative URLs are converted to absolute URLs
- Base URLs are configured per scraper
- URL encoding is handled automatically

## Testing the Scrapers

To test a specific scraper, use the API endpoint:
```bash
curl -X POST "http://localhost:8080/api/scraper/search" \
     -H "Content-Type: application/json" \
     -d '{"query": "your_search_term", "store": "store_name"}'
```

Where `store_name` can be:
- `bauhaus`
- `leroy`
- `bricodepot`
