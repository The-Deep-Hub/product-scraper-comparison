#!/bin/bash

echo "Testing search endpoint..."
curl -v -X POST http://localhost:8080/api/scraper/search \
  -H "Content-Type: application/json" \
  -d '{"query": "martillo", "store": null}'
echo -e "\n" 