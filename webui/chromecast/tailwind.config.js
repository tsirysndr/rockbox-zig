/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ['./src/**/*.{js,jsx,ts,tsx}'],
  theme: {
    extend: {
      fontFamily: {
        'rockford-light': ['RockfordSansLight', 'sans-serif'],
        'rockford': ['RockfordSansRegular', 'sans-serif'],
        'rockford-medium': ['RockfordSansMedium', 'sans-serif'],
        'rockford-bold': ['RockfordSansBold', 'sans-serif'],
        'rockford-extrabold': ['RockfordSansExtraBold', 'sans-serif'],
      },
    },
  },
  plugins: [],
};
