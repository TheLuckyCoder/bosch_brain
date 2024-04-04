const { fontFamily } = require("tailwindcss/defaultTheme");

/** @type {import('tailwindcss').Config} */
module.exports = {
    content: ["./templates/**/*.html"],
    theme: {
        extend: {
            fontFamily: {
                sans: ["Inter var", ...fontFamily.sans],
            },
            scale: {
                '175': '1.75',
                '200': '2.00',
            }
        },
    },
};