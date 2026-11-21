import { NextResponse } from "next/server";

// Simple in-memory rate-limiter: IP -> timestamp[]
const ipRequests = new Map<string, number[]>();
const RATE_LIMIT_WINDOW_MS = 60 * 60 * 1000; // 1 hour
const MAX_REQUESTS_PER_WINDOW = 3;

function isRateLimited(ip: string): boolean {
  const now = Date.now();
  const timestamps = ipRequests.get(ip) || [];
  const validTimestamps = timestamps.filter((t) => now - t < RATE_LIMIT_WINDOW_MS);

  if (validTimestamps.length >= MAX_REQUESTS_PER_WINDOW) {
    ipRequests.set(ip, validTimestamps);
    return true;
  }

  validTimestamps.push(now);
  ipRequests.set(ip, validTimestamps);
  return false;
}

export async function POST(request: Request) {
  try {
    const ip = request.headers.get("x-forwarded-for")?.split(",")[0]?.trim() || "anonymous";

    if (isRateLimited(ip)) {
      return NextResponse.json(
        { error: "Too many requests. Please wait an hour before submitting another issue." },
        { status: 429 }
      );
    }

    const body = await request.json();
    const { title, category, description, snippet, contact, honeypot } = body;

    // Honeypot check: if hidden bot field is filled, silently succeed without creating an issue
    if (honeypot && String(honeypot).trim() !== "") {
      return NextResponse.json({
        success: true,
        issue_url: "https://github.com/kobaltgit/StatCraft/issues",
        issue_number: 0,
      });
    }

    if (!title || !description) {
      return NextResponse.json(
        { error: "Title and description are required." },
        { status: 400 }
      );
    }

    const token = process.env.STATCRAFT_GITHUB_TOKEN || process.env.GITHUB_TOKEN;

    // In local development or testing without token, return a simulated successful response
    if (!token) {
      console.warn("[StatCraft API] STATCRAFT_GITHUB_TOKEN not set. Simulating issue submission.");
      return NextResponse.json({
        success: true,
        issue_url: "https://github.com/kobaltgit/StatCraft/issues",
        issue_number: Math.floor(Math.random() * 900) + 100,
        simulated: true,
      });
    }

    const issueBody = `### 🔬 Community Report (via StatCraft Website)
* **Category:** ${category || "General Feedback"}
* **Reported by:** ${contact ? `\`${contact}\`` : "Anonymous Researcher"}
* **IP Client:** \`${ip.slice(0, 7)}***\`
* **Submitted at:** ${new Date().toUTCString()}

#### Description:
${description}

${snippet ? `#### Data / Syntax Snippet:\n\`\`\`text\n${snippet}\n\`\`\`\n` : ""}

---
*Reported automatically via the StatCraft Web Portal (Vercel Serverless Gateway)*
`;

    const githubRes = await fetch("https://api.github.com/repos/kobaltgit/StatCraft/issues", {
      method: "POST",
      headers: {
        Authorization: `Bearer ${token}`,
        Accept: "application/vnd.github+json",
        "Content-Type": "application/json",
        "User-Agent": "StatCraft-Website-Bot",
      },
      body: JSON.stringify({
        title: `[Community] ${title}`,
        body: issueBody,
        labels: ["community-report", "needs-triage"],
      }),
    });

    if (!githubRes.ok) {
      const errText = await githubRes.text();
      console.error("[StatCraft API] GitHub API Error:", errText);
      return NextResponse.json(
        { error: "Failed to create issue on GitHub API.", details: errText },
        { status: githubRes.status }
      );
    }

    const issueData = await githubRes.json();
    return NextResponse.json({
      success: true,
      issue_url: issueData.html_url,
      issue_number: issueData.number,
    });
  } catch (err: unknown) {
    console.error("[StatCraft API] Unexpected error:", err);
    return NextResponse.json(
      { error: "Internal server error occurred." },
      { status: 500 }
    );
  }
}
