import type { Config } from "tailwindcss";

export default {
  // Specify paths to all templates where Tailwind classes are used
  content: [
    "./src/pages/**/*.{js,ts,jsx,tsx,mdx}",
    "./src/components/**/*.{js,ts,jsx,tsx,mdx}",
    "./src/app/**/*.{js,ts,jsx,tsx,mdx}",
    "./src/app/globals.css", // Include globals.css here
  ],
  theme: {
    extend: {
      colors: {
        background: "var(--background)", // Dynamic background color
        foreground: "var(--foreground)", // Dynamic text color
        primary: "#34A853", // Green for primary actions
        secondary: "#FF0000", // Red for warnings or alerts
        accent: "#FFC107", // Yellow for highlights
        muted: "#6C757D", // Muted or subdued text
      },
      fontFamily: {
        sans: ["Geist Sans", "sans-serif"], // General text
        mono: ["Geist Mono", "monospace"], // Technical text or code
      },
      screens: {
        xs: "480px", // Custom breakpoint for extra-small devices
      },
    },
  },
  darkMode: "class", // Enable dark mode based on a CSS class
  plugins: [
    require("@tailwindcss/forms"), // Simplify form styling
    require("@tailwindcss/aspect-ratio"), // Maintain aspect ratios for media
  ],
} satisfies Config;
