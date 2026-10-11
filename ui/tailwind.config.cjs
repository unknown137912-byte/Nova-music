/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/**/*.{html,js,svelte,ts}",
    "./node_modules/flowbite-svelte/**/*.{html,js,svelte,ts}"
  ],
  theme: {
    extend: {
      colors: {
        // NOVA Design System Colors
        'primary': {
          '50': '#f8f9fa',
          '100': '#e9ecef',
          '200': '#dee2e6',
          '300': '#adb5bd',
          '400': '#6c757d',
          '500': '#495057',
          '600': '#343a40',
          '700': '#212529',
          '800': '#1a1d20',
          '900': '#121316'
        },
        'accent': {
          '50': '#f3e8ff',
          '100': '#e9d5ff',
          '200': '#d8b4fe',
          '300': '#c084fc',
          '400': '#a855f7',
          '500': '#9333ea',
          '600': '#7c3aed',
          '700': '#6d28d9',
          '800': '#5b21b6',
          '900': '#4c1d95'
        }
      },
      fontFamily: {
        'sans': ['Inter', 'system-ui', 'sans-serif'],
        'display': ['SF Pro Display', 'system-ui', 'sans-serif'],
        'body': ['SF Pro Text', 'system-ui', 'sans-serif'],
        'mono': ['SF Mono', 'Menlo', 'monospace']
      },
      borderRadius: {
        'sm': '4px',
        'md': '8px',
        'lg': '12px',
        'xl': '16px',
        '2xl': '24px'
      },
      spacing: {
        '1': '4px',
        '2': '8px',
        '3': '12px',
        '4': '16px',
        '5': '20px',
        '6': '24px',
        '7': '28px',
        '8': '32px',
        '9': '36px',
        '10': '40px'
      },
      boxShadow: {
        'sm': '0 1px 2px rgba(0, 0, 0, 0.05)',
        'md': '0 4px 6px rgba(0, 0, 0, 0.1)',
        'lg': '0 10px 15px rgba(0, 0, 0, 0.1)',
        'xl': '0 20px 25px rgba(0, 0, 0, 0.1)',
        '2xl': '0 25px 50px rgba(0, 0, 0, 0.15)'
      }
    }
  },
  plugins: []
};