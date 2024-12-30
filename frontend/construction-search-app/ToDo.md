# To-Do List

## Priority: Endpoint Simulation
- [ ] **Define and simulate `/api/products` endpoint** in the frontend.
- [ ] **Finalize JSON structure** for the endpoint and document it for the backend developer.
- [ ] **Share API expectations** and required error handling with the backend team.
- [ ] **Test simulated API integration** with the frontend.

---

## Refactoring
- [ ] **Move reusable code into modular components** in the `components` folder:
  - [ ] Filter Panel Component.
  - [ ] Result Card Component.
  - [ ] Search Bar Component.
- [ ] **Optimize `layout` and `page` files** by utilizing the newly created components.

---

## Styling and UI Enhancements
- [ ] **Apply a cohesive color palette** to the frontend.
- [ ] **Add animations** for:
  - [ ] Buttons.
  - [ ] Result cards (hover effect with background change).
- [ ] **Fix result card height**:
  - [ ] Set a fixed height for cards to avoid layout inconsistencies.
  - [ ] Ensure cards look consistent across all results.
- [ ] **Adjust main content container width** for wider screens.

---

## Functionality Improvements
- [ ] **Sort by price (ascending/descending)** in the filter panel.
- [ ] **Add a category filter** (checkboxes for providers: Bauhaus, Leroy Merlin, etc.).
- [ ] **Handle missing data**:
  - [ ] Show product title as a placeholder description if `description` is missing.
- [ ] **Improve user feedback for empty searches**:
  - [ ] Display a user-friendly message like: "Please enter a search term to find products."

---

## Responsive Design
- [ ] **Adjust design for various devices**:
  - [ ] **iPhone 8 / Redmi 9** (393 x 851).
  - [ ] **iPad Mini (Vertical)** (768 x 1024).
  - [ ] **iPad Pro 12.9" (Vertical)** (1024 x 1366).
  - [ ] **Laptop (1440 x 900)**.
  - [ ] **Full HD Screen (1920 x 1080)**.
- [ ] **Test responsiveness** and adjust the layout for breakpoints.

---

## Documentation
- [ ] **Document the proposed API endpoint** for the backend developer.
- [ ] **Write detailed usage documentation** for:
  - [ ] Filters (including new features).
  - [ ] Search functionality.
  - [ ] Expected API integration behavior.

---

# Proposed GitHub Issues

### Refactoring
1. **Refactor Components**  
   Move reusable code from `layout` and `page` files into modular components.

---

### Styling
2. **Apply Styling**  
   Use a defined color palette and add animations for UI/UX improvement.

3. **Fix Result Card Height**  
   Adjust card heights to a consistent size for a cleaner layout.

4. **Adjust Main Content Width**  
   Make the main content container adapt better on wider screens.

---

### Features
5. **Sort by Price in Filter Panel**  
   Add options to sort products by ascending or descending price.

6. **Category Filter**  
   Add a checkbox-based category filter for predefined providers (e.g., Leroy Merlin).

7. **Handle Missing Data**  
   Provide fallback values for missing data (e.g., use product title if no description exists).

---

### Responsive Design
8. **Responsive Layout**  
   Ensure the site is responsive and adjusts properly to popular device resolutions.

---

### API Integration
9. **Simulate API Endpoint**  
   Simulate the `/api/products` endpoint with a finalized JSON structure and test its integration in the frontend.

10. **Document API Expectations**  
    Write detailed documentation for the backend developer outlining:  
    - JSON structure.  
    - Expected behavior.
    - Error handling for missing or incomplete data.  
    - Pagination behavior:
      - Include `page` and `limit` query parameters.
      - Response structure to include total number of pages, current page, and results. 