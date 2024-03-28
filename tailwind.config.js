/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./web/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        black: "#000",
        offWhite: "#EFEEEB",
        white: "#FFF",
        green: "rgb(6,176, 113)",
        lime: "rgb(170, 240, 95)",
        orange: "rgb(245, 114, 49)",
        lavendar: "rgb(172, 155, 245)",
        blue: "rgb(87, 84, 246)",
        dustPink: "rgb(231, 178, 178)",
      },
      fontFamily: {
        gasoek: ["Gasoek One", "sans-serif"],
        mono: ["DM Mono", "monospace"],
        sans: ["DM Sans", "sans-serif"],
      },
    },
  },
  plugins: [require("@tailwindcss/forms")],
};
