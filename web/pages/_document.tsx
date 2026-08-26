import { Html, Head, Main, NextScript } from "next/document";

const themeInitScript = `
(function() {
  try {
    var storedTheme = localStorage.getItem('theme');
    var prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    if (storedTheme === 'dark' || (!storedTheme && prefersDark)) {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
  } catch (e) {}
})();
`;

export default function Document() {
  return (
    <Html lang="en">
      <Head />
      <body>
        <script dangerouslySetInnerHTML={{ __html: themeInitScript }} />
        <Main />
        <NextScript />
      </body>
    </Html>
  );
}
