const {fontFamily} = require("tailwindcss/defaultTheme");

/** @type {import('tailwindcss').Config} */
module.exports = {
    content: ["./templates/**/*.html"],
    plugins: [require('daisyui')],
    theme: {
        extend: {
            fontFamily: {
                sans: ["Inter var", ...fontFamily.sans],
            },
        },
    },
    daisyui: {
        themes: ["dracula"],
    },
};