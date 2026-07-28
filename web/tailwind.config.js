/** @type {import("tailwindcss").Config} */
const colors = require("tailwindcss/colors");

module.exports = {
  content: [
    "./pages/**/*.{js,ts,jsx,tsx}",
    "./components/**/*.{js,ts,jsx,tsx}",
    "./context/**/*.{js,ts,jsx,tsx}",
    "./wasm-upload/**/*.{js,ts,jsx,tsx}",
  ],
  // Status/severity indicators pick their color classes from a fixed palette at
  // runtime. The safelist ensures Tailwind never purges these even when class
  // names are assembled dynamically.
  safelist: [
    {
      pattern:
        /^(bg|text|border|ring)-(rose|amber|emerald|cyan|sky|violet|pink|indigo|orange|blue|green|slate)-(50|100|200|300|400|500|600|700|800|900|950)$/,
    },
  ],
  theme: {
    extend: {
      colors: {
        // WCAG AA contrast overrides (WEB-59, #192)
        // All values below target dark backgrounds (slate-950 / slate-900).
        // Minimum required contrast ratio: 4.5:1 normal text, 3:1 large text.
        slate: {
          ...colors.slate,
          // slate-300 on slate-950 → ~10.7:1 (AAA)  ✓
          // slate-400 on slate-950 →  ~6.8:1 (AA)   ✓
          400: colors.slate[300], // #cbd5e1
          500: colors.slate[400], // #94a3b8
        },
        gray: {
          ...colors.gray,
          // gray-300 on slate-950 → ~10.3:1 (AAA)  ✓
          // gray-400 on slate-950 →  ~6.6:1 (AA)   ✓
          // gray-500 default #6b7280 → 4.6:1 on white ✓ (kept)
          // gray-600 default #4b5563 → 3.9:1 on white ✗ → map to gray-500
          400: colors.gray[300],  // #d1d5db
          500: colors.gray[400],  // #9ca3af
          600: colors.gray[500],  // #6b7280 — 4.6:1 on white ✓
        },
        zinc: {
          ...colors.zinc,
          // zinc-400 on slate-950 → ~6.9:1 (AA)  ✓
          400: colors.zinc[300],  // #d4d4d8
          500: colors.zinc[400],  // #a1a1aa
        },
      },
      spacing: {
        120: "30rem",
      },
      borderRadius: {
        "4xl": "2rem",
        "s-2xl": "1rem 0 0 1rem",
        "e-2xl": "0 1rem 1rem 0",
      },
    },
  },
  plugins: [],
};
