/** @type {import('tailwindcss').Config} */
module.exports = {
  content: { 
    files: ["./**/*.rs"],
  },
  theme: {
    extend: {
      maxWidth: {
        "1/4": "25%",
        "1/2": "50%",
        "3/4": "75%",
      },
    },
  },
  plugins: [],
  variants: {
    "overflow": ["hover", "group-hover"],
    "visibility": ["hover", "group-hover"],
  }
}