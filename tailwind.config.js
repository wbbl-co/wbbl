/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./web/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        black: "#000",
        offWhite: "#EFEEEB",
        white: "#FFF",
        green: "#2C4EFF",
        lime: "#FF5C01",
        orange: "#FFD92D",
        lavendar: "#24D480",
        blue: "rgb(87, 84, 246)",
        dustPink: "#AB9BF2",
      },
      fontFamily: {
        gasoek: ["GasoekOne", "sans-serif"],
        mono: ["DMMono", "monospace"],
        sans: ["DMMono", "monospace"],
      },
    },
  },
  plugins: [require("@tailwindcss/forms")],
};
