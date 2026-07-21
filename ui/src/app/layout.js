import "./globals.css";
import ConsoleLauncher from "./ConsoleLauncher";

export const metadata = {
  title: "Star — Desktop Intelligence",
  description: "Chat with Star, your local AI companion",
};

export default function RootLayout({ children }) {
  return (
    <html lang="en">
      <head>
        <link
          href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;600;700&display=swap"
          rel="stylesheet"
        />
      </head>
      <body>
        {children}
        <ConsoleLauncher />
      </body>
    </html>
  );
}
