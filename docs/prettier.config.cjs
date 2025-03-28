/** @type {import("prettier").Config} */
module.exports = {
  // Any other Prettier options you like:
  semi: true,
  singleQuote: false,
  // This is crucial: it loads the plugin that sorts Tailwind classes
  plugins: ["prettier-plugin-tailwindcss"],
};
