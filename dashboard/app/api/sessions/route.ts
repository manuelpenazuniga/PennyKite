import { NextResponse } from "next/server";
import { proxyUrl } from "@/lib/proxy";
import type { SessionsResponse } from "@/lib/types";

export const dynamic = "force-dynamic";

export async function GET() {
  try {
    const res = await fetch(proxyUrl("/api/sessions"), { cache: "no-store" });
    if (!res.ok) {
      return NextResponse.json(
        { error: "proxy_sessions_error", reason: `proxy returned HTTP ${res.status}` },
        { status: 502 },
      );
    }

    const data = (await res.json()) as SessionsResponse;
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
