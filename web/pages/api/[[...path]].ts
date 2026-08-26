import type { NextApiRequest, NextApiResponse } from "next";

const BACKEND_URL = (
  process.env.API_URL ||
  process.env.NEXT_PUBLIC_API_URL ||
  "http://localhost:8080"
).replace(/\/+$/, "");

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
  try {
    const targetUrl = `${BACKEND_URL}${req.url}`;

    const headers: Record<string, string> = {};
    for (const [key, value] of Object.entries(req.headers)) {
      if (typeof value === "string" && key !== "host" && key !== "content-length") {
        headers[key] = value;
      }
    }
    headers["host"] = new URL(BACKEND_URL).host;

    const fetchOptions: RequestInit = {
      method: req.method,
      headers,
    };

    if (req.method !== "GET" && req.method !== "HEAD") {
      fetchOptions.body = JSON.stringify(req.body);
    }

    const response = await fetch(targetUrl, fetchOptions);

    res.status(response.status);
    response.headers.forEach((value, key) => {
      if (
        key !== "transfer-encoding" &&
        key !== "content-encoding" &&
        key !== "content-length"
      ) {
        res.setHeader(key, value);
      }
    });

    const responseBody = await response.text();
    res.send(responseBody);
  } catch (error) {
    console.error("API proxy error:", error);
    res.status(500).json({ message: "Internal proxy error" });
  }
}
