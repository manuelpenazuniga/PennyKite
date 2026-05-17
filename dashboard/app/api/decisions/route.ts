import { NextResponse } from "next/server";
import type { Decision, SessionSummary } from "@/lib/types";

const DEFAULT_PROXY_URL = "http://127.0.0.1:8787";

interface FeedResponse {
  summary: SessionSummary;
  decisions: Decision[];
}

export const dynamic = "force-dynamic";

export async function GET() {
  const proxyUrl = process.env.PENNYKITE_PROXY_URL ?? DEFAULT_PROXY_URL;
  const feedUrl = new URL("/api/decisions", proxyUrl);

  try {
    const res = await fetch(feedUrl, { cache: "no-store" });
    if (!res.ok) {
      return NextResponse.json(
        { error: "proxy_feed_error", reason: `proxy returned HTTP ${res.status}` },
        { status: 502 },
      );
    }

    const data = (await res.json()) as FeedResponse;
    return NextResponse.json(data, {
      headers: { "cache-control": "no-store" },
    });
  } catch (e) {
    return NextResponse.json(
      {
        error: "proxy_unreachable",
        reason: e instanceof Error ? e.message : "fetch failed",
      },
      { status: 502 },
    );
  }
}
